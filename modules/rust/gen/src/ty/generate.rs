use super::{BuiltIn, Description, Primitive, Type};
use crate::{Context, Error, Generate, Module, Referenced};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use rdf_types::Vocabulary;
use shelves::Ref;
use treeldr::{BlankIdIndex, IriIndex, TId};

mod json_ld;
mod rdf;

pub fn type_ident_of_name(name: &treeldr::Name) -> proc_macro2::Ident {
	quote::format_ident!("{}", name.to_pascal_case())
}

pub fn field_ident_of_name(name: &treeldr::Name) -> proc_macro2::Ident {
	let mut name = name.to_snake_case();
	if matches!(name.as_str(), "type") {
		name.push('_')
	}

	quote::format_ident!("{}", name)
}

pub fn variant_ident_of_name(name: &treeldr::Name) -> proc_macro2::Ident {
	quote::format_ident!("{}", name.to_pascal_case())
}

pub fn doc_attribute(
	label: Option<&str>,
	doc: &treeldr::StrippedDocumentation,
) -> Vec<TokenStream> {
	let mut content = String::new();

	if let Some(label) = label {
		content.push_str(label)
	}

	if let Some(short) = doc.short_description() {
		if !content.is_empty() {
			content.push_str("\n\n");
		}

		content.push_str(short)
	}

	if let Some(long) = doc.long_description() {
		if !content.is_empty() {
			content.push_str("\n\n");
		}

		content.push_str(long)
	}

	content
		.lines()
		.map(|line| {
			quote::quote! {
				#[doc = #line]
			}
		})
		.collect()
}

impl<M> Generate<M> for BuiltIn {
	fn generate<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: Option<Ref<Module>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		match self {
			Self::Required(item) => {
				item.generate(context, scope, tokens)?;
			}
			Self::Option(item) => {
				let item = item.with(context, scope).into_tokens()?;
				tokens.extend(quote! { Option<#item> })
			}
			Self::Vec(item) => {
				let item = item.with(context, scope).into_tokens()?;
				tokens.extend(quote! { Vec<#item> })
			}
			Self::BTreeSet(item) => {
				let item = item.with(context, scope).into_tokens()?;
				tokens.extend(quote! { std::collections::BTreeSet<#item> })
			}
			Self::OneOrMany(item) => {
				let item = item.with(context, scope).into_tokens()?;
				tokens.extend(quote! { ::treeldr_rust_prelude::OneOrMany<#item> })
			}
		}

		Ok(())
	}
}

impl<M> Generate<M> for Referenced<BuiltIn> {
	fn generate<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: Option<Ref<Module>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		match self.0 {
			BuiltIn::Required(item) => {
				Referenced(item).generate(context, scope, tokens)?;
			}
			BuiltIn::Option(item) => {
				let item_ref = Referenced(item).with(context, scope).into_tokens()?;
				tokens.extend(quote! { Option<#item_ref> })
			}
			BuiltIn::Vec(item) => {
				let item = item.with(context, scope).into_tokens()?;
				tokens.extend(quote! { &[#item] })
			}
			BuiltIn::BTreeSet(item) => {
				let item = item.with(context, scope).into_tokens()?;
				tokens.extend(quote! { &std::collections::BTreeSet<#item> })
			}
			BuiltIn::OneOrMany(item) => {
				let item = item.with(context, scope).into_tokens()?;
				tokens.extend(quote! { &::treeldr_rust_prelude::OneOrMany<#item> })
			}
		}

		Ok(())
	}
}

impl<M> Generate<M> for Type {
	fn generate<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: Option<Ref<Module>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		let doc = doc_attribute(self.label(), self.documentation());

		match &self.desc {
			Description::Alias(ident, alias_ref) => {
				let alias = alias_ref.with(context, scope).into_tokens()?;

				tokens.extend(quote! {
					#(#doc)*
					pub type #ident = #alias;
				})
			}
			Description::Struct(s) => {
				let ident = s.ident();
				let mut fields = Vec::with_capacity(s.fields().len());
				let mut required_inputs = Vec::new();
				let mut fields_init = Vec::new();
				let mut derives = vec![
					quote! { Clone },
					quote! { PartialEq },
					quote! { Eq },
					quote! { PartialOrd },
					quote! { Ord },
					quote! { Debug },
				];

				for field in s.fields() {
					fields.push(field.with(context, scope).into_tokens()?);

					let field_ident = field.ident();
					let init = if field.ty(context).impl_default(context) {
						quote! {
							Default::default()
						}
					} else {
						let ty = field.layout().with(context, scope).into_tokens()?;
						required_inputs.push(quote! {
							#field_ident : #ty,
						});

						quote! {
							#field_ident
						}
					};

					fields_init.extend(quote! { #field_ident : #init, })
				}

				if required_inputs.is_empty() {
					derives.push(quote! { Default });
				}

				tokens.extend(quote! {
					#(#doc)*
					#[derive(#(#derives),*)]
					pub struct #ident {
						#(#fields),*
					}
				});

				if !required_inputs.is_empty() {
					tokens.extend(quote! {
						impl #ident {
							pub fn new(#(#required_inputs)*) -> Self {
								Self {
									#(#fields_init)*
								}
							}
						}
					})
				}

				tokens.extend(rdf::structure_reader(context, s, ident));
				tokens.extend(json_ld::structure_builder(context, s, ident))
			}
			Description::Enum(e) => {
				let ident = e.ident();
				let mut variants = Vec::with_capacity(e.variants().len());

				for variant in e.variants() {
					variants.push(variant.with(context, scope).into_tokens()?)
				}

				tokens.extend(quote! {
					#(#doc)*
					#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
					pub enum #ident {
						#(#variants),*
					}
				});

				tokens.extend(rdf::enum_reader(context, e, ident));
				tokens.extend(json_ld::enum_builder(context, e, ident))
			}
			_ => (),
		}

		Ok(())
	}
}

impl<M> Generate<M> for TId<treeldr::Layout> {
	fn generate<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: Option<Ref<Module>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		let ty = context
			.layout_type(*self)
			.expect("undefined generated layout");
		match &ty.desc {
			Description::Never => {
				tokens.extend(quote! { ! });
				Ok(())
			}
			Description::Primitive(p) => p.generate(context, scope, tokens),
			Description::Alias(ident, _) => {
				let path = ty
					.path(context, ident.clone())
					.ok_or(Error::UnreachableType(*self))?;
				context.module_path(scope).to(&path).to_tokens(tokens);
				Ok(())
			}
			Description::Struct(s) => {
				let path = ty
					.path(context, s.ident().clone())
					.ok_or(Error::UnreachableType(*self))?;
				context.module_path(scope).to(&path).to_tokens(tokens);
				Ok(())
			}
			Description::Enum(e) => {
				let path = ty
					.path(context, e.ident().clone())
					.ok_or(Error::UnreachableType(*self))?;
				context.module_path(scope).to(&path).to_tokens(tokens);
				Ok(())
			}
			Description::Reference => {
				let id_ty = context.ident_type();
				tokens.extend(quote! { #id_ty });
				Ok(())
			}
			Description::BuiltIn(b) => b.generate(context, scope, tokens),
		}
	}
}

impl<M> Generate<M> for Referenced<TId<treeldr::Layout>> {
	fn generate<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: Option<Ref<Module>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		let ty = context
			.layout_type(self.0)
			.expect("undefined generated layout");
		match &ty.desc {
			Description::Never => {
				tokens.extend(quote! { ! });
				Ok(())
			}
			Description::Primitive(p) => Referenced(*p).generate(context, scope, tokens),
			Description::Alias(ident, _) => {
				let abs_path = ty
					.path(context, ident.clone())
					.ok_or(Error::UnreachableType(self.0))?;
				let path = context.module_path(scope).to(&abs_path);
				tokens.extend(quote! { &#path });
				Ok(())
			}
			Description::Struct(s) => {
				let abs_path = ty
					.path(context, s.ident().clone())
					.ok_or(Error::UnreachableType(self.0))?;
				let path = context.module_path(scope).to(&abs_path);
				tokens.extend(quote! { &#path });
				Ok(())
			}
			Description::Enum(e) => {
				let abs_path = ty
					.path(context, e.ident().clone())
					.ok_or(Error::UnreachableType(self.0))?;
				let path = context.module_path(scope).to(&abs_path);
				tokens.extend(quote! { &#path });
				Ok(())
			}
			Description::Reference => {
				tokens.extend(quote! { ::iref::Iri });
				Ok(())
			}
			Description::BuiltIn(b) => Referenced(*b).generate(context, scope, tokens),
		}
	}
}

impl<M> Generate<M> for treeldr::layout::Primitive {
	fn generate<V>(
		&self,
		_context: &Context<V, M>,
		_scope: Option<Ref<Module>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		tokens.extend(match self {
			Self::Boolean => quote! { bool },
			Self::Integer => quote! { i32 },
			Self::UnsignedInteger => quote! { u32 },
			Self::Float => quote! { f32 },
			Self::Double => quote! { f64 },
			Self::String => quote! { ::std::string::String },
			Self::Date => quote! { ::chrono::Date<::chrono::Utc> },
			Self::DateTime => quote! { ::chrono::DateTime<::chrono::Utc> },
			Self::Time => quote! { ::chrono::NaiveTime },
			Self::Url => quote! { ::iref::IriBuf },
			Self::Uri => quote! { ::iref::IriBuf },
			Self::Iri => quote! { ::iref::IriBuf },
		});

		Ok(())
	}
}

impl<M> Generate<M> for Referenced<treeldr::layout::Primitive> {
	fn generate<V>(
		&self,
		_context: &Context<V, M>,
		_scope: Option<Ref<Module>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		tokens.extend(match self.0 {
			Primitive::Boolean => quote! { bool },
			Primitive::Integer => quote! { i32 },
			Primitive::UnsignedInteger => quote! { u32 },
			Primitive::Float => quote! { f32 },
			Primitive::Double => quote! { f64 },
			Primitive::String => quote! { &str },
			Primitive::Date => quote! { ::chrono::Date },
			Primitive::DateTime => quote! { ::chrono::DateTime },
			Primitive::Time => quote! { ::chrono::NaiveTime },
			Primitive::Url => quote! { ::iref::Iri },
			Primitive::Uri => quote! { ::iref::Iri },
			Primitive::Iri => quote! { ::iref::Iri },
		});

		Ok(())
	}
}
