use proc_macro2::{Span, TokenStream};
use quote::quote;

use crate::attributes;

use self::attributes::{Attributes, TypeExprAttributes};

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("unsupported `union` type")]
	Union(Span),

	#[error(transparent)]
	Attribute(#[from] attributes::Error),
}

impl Error {
	pub fn span(&self) -> Span {
		match self {
			Self::Union(s) => *s,
			Self::Attribute(e) => e.span(),
		}
	}
}

pub fn derive(input: syn::DeriveInput) -> Result<TokenStream, Error> {
	let ident = &input.ident;
	let name = ident.to_string();

	let has_resource = !input.generics.params.is_empty();

	let resource_arg = has_resource.then_some(quote! { <R> });

	let attrs = Attributes::parse_attributes(&input.attrs)?;
	let predicates = &attrs.deserialize_bounds;

	let body = match input.data {
		syn::Data::Struct(s) => derive_fields(ident, has_resource, None, &s.fields)?,
		syn::Data::Enum(e) => {
			let count = e.variants.len();

			let variants_idents = e.variants.iter().map(|v| &v.ident);
			let variants_names = e.variants.iter().map(|v| v.ident.to_string());

			let variants_cases: Vec<_> = e
				.variants
				.iter()
				.map(|v| {
					let variant_ident = &v.ident;

					let de_fields =
						derive_fields(ident, has_resource, Some(variant_ident), &v.fields)?;

					Ok(quote! {
						Discriminant::#variant_ident => {
							#de_fields
						}
					})
				})
				.collect::<Result<_, Error>>()?;

			let cases_u64 = e.variants.iter().enumerate().map(|(i, v)| {
				let i = i as u64;
				let variant_ident = &v.ident;

				quote! {
					#i => Ok(Discriminant::#variant_ident)
				}
			});

			let cases_str = e.variants.iter().map(|v| {
				let variant_name = v.ident.to_string();
				let variant_ident = &v.ident;

				quote! {
					#variant_name => Ok(Discriminant::#variant_ident)
				}
			});

			let cases_bytes = e.variants.iter().map(|v| {
				let variant_name = v.ident.to_string();
				let variant_bytes = syn::LitByteStr::new(variant_name.as_bytes(), v.ident.span());
				let variant_ident = &v.ident;

				quote! {
					#variant_bytes => Ok(Discriminant::#variant_ident)
				}
			});

			quote! {
				const VARIANTS: [&str; #count] = [
					#(#variants_names),*
				];

				struct Visitor<'a, R> {
					index: &'a crate::rdf::ResourceBackwardIndex<R>
				}

				impl<'a, 'de, R> ::serde::de::Visitor<'de> for Visitor<'a, R> where #(#predicates),* {
					type Value = #ident #resource_arg;

					fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
						write!(formatter, "enum value")
					}

					fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
					where
						A: ::serde::de::EnumAccess<'de>
					{
						enum Discriminant {
							#(#variants_idents),*
						}

						struct DiscriminantVisitor;

						impl<'de> ::serde::de::Visitor<'de> for DiscriminantVisitor {
							type Value = Discriminant;

							fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
								write!(formatter, "variant identifier")
							}

							fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
							where
								E: ::serde::de::Error
							{
								match v {
									#(#cases_u64,)*
									_ => Err(::serde::de::Error::invalid_value(::serde::de::Unexpected::Unsigned(v), &"variant index"))
								}
							}

							fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
							where
								E: ::serde::de::Error
							{
								match v {
									#(#cases_str,)*
									_ => Err(::serde::de::Error::unknown_variant(v, &VARIANTS))
								}
							}

							fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
							where
								E: ::serde::de::Error
							{
								match v {
									#(#cases_bytes,)*
									_ => {
										let v = String::from_utf8_lossy(v);
										Err(::serde::de::Error::unknown_variant(&v, &VARIANTS))
									}
								}
							}
						}

						impl<'de> ::serde::de::Deserialize<'de> for Discriminant {
							fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
							where
								D: ::serde::Deserializer<'de>
							{
								deserializer.deserialize_identifier(DiscriminantVisitor)
							}
						}

						let (discriminant, variant) = data.variant::<Discriminant>()?;

						match discriminant {
							#(#variants_cases),*
						}
					}
				}

				deserializer.deserialize_enum(#name, &VARIANTS, Visitor {
					index
				})
			}
		}
		syn::Data::Union(u) => return Err(Error::Union(u.union_token.span)),
	};

	Ok(quote! {
		impl<'de, R> crate::rdf::serde::DeserializeIndexed<'de, R> for #ident #resource_arg where #(#predicates),* {
			fn deserialize_indexed<D>(
				index: &crate::rdf::ResourceBackwardIndex<R>,
				deserializer: D
			) -> Result<Self, D::Error> where D: ::serde::Deserializer<'de> {
				#body
			}
		}
	})
}

fn derive_fields(
	ident: &syn::Ident,
	has_resource: bool,
	variant_ident: Option<&syn::Ident>,
	fields: &syn::Fields,
) -> Result<TokenStream, Error> {
	let resource_arg = has_resource.then_some(quote! { <R> });

	let variant_ext = variant_ident.map(|i| {
		quote! {
			:: #i
		}
	});

	match fields {
		syn::Fields::Unit => Ok(quote! {
			::serde::de::VariantAccess::unit_variant(variant)?;
			Ok(#ident #variant_ext)
		}),
		syn::Fields::Unnamed(fields) => {
			if fields.unnamed.len() == 1 {
				let v = fields.unnamed.first().unwrap();
				let attrs = TypeExprAttributes::parse_attributes(&v.attrs)?;
				let seed = deserialize_seed(&attrs, &v.ty);

				if variant_ident.is_some() {
					Ok(quote! {
						::serde::de::VariantAccess::newtype_variant_seed(variant, #seed)
							.map(#ident #variant_ext)
					})
				} else {
					// quote! {
					// 	deserializer::deserialize_newtype_struct::<#ty>(variant)
					// 		.map(#ident #variant_ext)
					// }
					todo!()
				}
			} else {
				let args: Vec<_> = fields
					.unnamed
					.iter()
					.map(|v| {
						let attrs = TypeExprAttributes::parse_attributes(&v.attrs)?;
						let seed = deserialize_seed(&attrs, &v.ty);
						Ok(quote! {
							seq.next_element_seed(#seed)?
						})
					})
					.collect::<Result<_, Error>>()?;

				let visit = if variant_ident.is_some() {
					quote! {
						::serde::de::VariantAccess::tuple_variant(variant, TupleVisitor {
							index
						})
					}
				} else {
					quote! {
						deserializer.deserialize_tuple_struct(variant, TupleVisitor {
							index
						})
					}
				};

				Ok(quote! {
					struct TupleVisitor<'a, R> {
						index: &'a crate::rdf::ResourceBackwardIndex<R>
					}

					impl<'a, 'de, R> ::serde::de::Visitor<'de> for TupleVisitor<'a, R> {
						type Value = #ident #resource_arg;

						fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> {
							Ok(#ident #variant_ext (#(#args),*))
						}
					}

					#visit
				})
			}
		}
		syn::Fields::Named(fields) => {
			let visit = if variant_ident.is_some() {
				quote! {
					::serde::de::VariantAccess::struct_variant(variant, FIELDS, StructVisitor)
				}
			} else {
				quote! {
					deserialize.tuple_struct(FIELDS, StructVisitor)
				}
			};

			Ok(quote! {
				struct StructVisitor<'a, R> {
					index: &'a crate::rdf::ResourceBackwardIndex<R>
				}

				impl<'a, 'de, R> ::serde::de::Visitor<'de> for TupleVisitor<'a, R> {
					type Value = #ident #resource_arg;

					fn visit_struct<A>(self, mut seq: A) -> Result<Self::Value, A::Error> {
						// Ok(#ident #variant_ext (#(#args),*))
						todo!()
					}
				}

				#visit
			})
		}
	}
}

fn deserialize_seed(attrs: &TypeExprAttributes, ty: &syn::Type) -> TokenStream {
	match &attrs.with {
		Some(id) => {
			quote! {
				#id::deserialize_seed(self.index)
			}
		}
		None => {
			quote! {
				self.index.deserialize_seed::<#ty>()
			}
		}
	}
}
