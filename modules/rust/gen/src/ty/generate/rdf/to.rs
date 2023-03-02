use std::collections::BTreeSet;

use proc_macro2::{Ident, TokenStream};
use quote::{quote, format_ident};
use rdf_types::Vocabulary;
use treeldr::{TId, Id, IriIndex, BlankIdIndex, vocab::Primitive};

use crate::{ty::{generate::GenerateFor, structure::Struct, enumeration::Enum, self, params::ParametersValues}, Generate, Error, GenerateList};

/// `RdfTriples` trait implementation.
pub struct RdfTriplesImpl;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Bound {
	AsLiteral(Primitive)
}

impl<M> Generate<M> for Bound {
	fn generate<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &crate::Context<V, M>,
		scope: Option<shelves::Ref<crate::Module>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		match self {
			Self::AsLiteral(p) => {
				let ty = p.generate_with(context, scope).into_tokens()?;
				tokens.extend(quote!(#ty: ::treeldr_rust_prelude::rdf::AsLiteral<N, V>));
				Ok(())
			}
		}
	}
}

fn triples_and_values_iterator_name_from(
	ident: &Ident
) -> Ident {
	format_ident!("{ident}TriplesAndValues")
}

fn triples_and_values_iterator_of<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>, M>(
	context: &crate::Context<V, M>,
	scope: Option<shelves::Ref<crate::Module>>,
	layout: TId<treeldr::Layout>,
	lifetime: TokenStream,
	mut bounds: impl FnMut(Bound)
) -> Result<TokenStream, Error> {
	let ty = context.layout_type(layout).unwrap();

	match ty.description() {
		ty::Description::Never => {
			Ok(quote!(::treeldr_rust_prelude::rdf::iter::Empty<V>))
		}
		ty::Description::Alias(_, target) => {
			triples_and_values_iterator_of(context, scope, *target, lifetime, bounds)
		}
		ty::Description::Primitive(p) => {
			bounds(Bound::AsLiteral(*p));
			let p = p.generate_with(context, scope).into_tokens()?;
			Ok(quote!(::treeldr_rust_prelude::rdf::iter::ValuesOnly<::treeldr_rust_prelude::rdf::iter::LiteralValue<'a, #p, I, V>>))
		}
		ty::Description::BuiltIn(b) => {
			match b {
				ty::BuiltIn::BTreeSet(item_layout) => {
					let item_ty = context.layout_type(*item_layout).unwrap();
					let mut path = context.module_path(scope).to(&context.parent_module_path(item_ty.module()).unwrap());
					path.push(item_ty.ident());
					Ok(quote!(::treeldr_rust_prelude::rdf::iter::Flatten<::std::collection::btree_set::Iter<#lifetime, #path>>))
				}
				ty::BuiltIn::OneOrMany(_) => {
					todo!()
				}
				ty::BuiltIn::Option(item_layout) => {
					let inner = triples_and_values_iterator_of(context, scope, *item_layout, lifetime, bounds)?;
					Ok(quote!(::treeldr_rust_prelude::rdf::iter::Optional<#inner>))
				}
				ty::BuiltIn::Required(item_layout) => {
					triples_and_values_iterator_of(context, scope, *item_layout, lifetime, bounds)
				}
				ty::BuiltIn::Vec(item_layout) => {
					let item_ty = context.layout_type(*item_layout).unwrap();
					let mut path = context.module_path(scope).to(&context.parent_module_path(item_ty.module()).unwrap());
					path.push(item_ty.ident());
					Ok(quote!(::treeldr_rust_prelude::rdf::iter::Flatten<::std::slice::Iter<#lifetime, #path>>))
				}
			}
		}
		ty::Description::Struct(s) => {
			let mut path = context.module_path(scope).to(&context.parent_module_path(ty.module()).unwrap());
			path.push(triples_and_values_iterator_name_from(s.ident()));
			Ok(quote!(#path))
		}
		ty::Description::Enum(e) => {
			let mut path = context.module_path(scope).to(&context.parent_module_path(ty.module()).unwrap());
			path.push(triples_and_values_iterator_name_from(e.ident()));
			Ok(quote!(#path))
		}
		ty::Description::Reference => {
			todo!("reference iterator")
		}
	}
}

impl<M> GenerateFor<Struct, M> for RdfTriplesImpl {
	fn generate<V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>>(
		&self,
		context: &crate::Context<V, M>,
		scope: Option<shelves::Ref<crate::Module>>,
		ty: &Struct,
		tokens: &mut proc_macro2::TokenStream,
	) -> Result<(), crate::Error> {
		let ident = ty.ident();
		let params_values = ParametersValues::new(quote!(N::Id));
		let params = ty.params().instantiate(&params_values);
		let iterator_ident = triples_and_values_iterator_name_from(ident);

		let mut iterator_fields = Vec::with_capacity(ty.fields().len());
		let mut iterator_fields_init = Vec::with_capacity(ty.fields().len());
		let mut iterator_id_init = None;
		let mut next = quote!(self.id_.take().map(::treeldr_rust_prelude::rdf_types::Object::Id).map(::treeldr_rust_prelude::rdf::TripleOrValue::Value));
		let mut bounds = BTreeSet::new();
		for field in ty.fields() {
			let field_ident = field.ident();
			if field.is_self() {
				iterator_id_init = Some(quote! {
					self.#field_ident.clone()
				})
			} else {
				let iter_ty = triples_and_values_iterator_of(context, scope, field.layout(), quote!('a), |b| { bounds.insert(b); })?;
				iterator_fields.push(quote! {
					#field_ident: #iter_ty
				});
				iterator_fields_init.push(quote! {
					#field_ident: self.#field_ident.unbound_rdf_triples_and_values(namespace, generator)
				});

				let mut prop_iri = None;
				if let Some(prop_id) = field.property() {
					if let Id::Iri(iri_index) = prop_id.id() {
						prop_iri = Some(iri_index);
					}
				}

				let map_prop_item = match prop_iri {
					Some(iri_index) => {
						let prop_iri = context.vocabulary().iri(&iri_index).unwrap().into_str();
						quote! {
							.map(|item| match item {
								::treeldr_rust_prelude::rdf::TripleOrValue::Triple(triple) => {
									treeldr_rust_prelude::rdf::TripleOrValue::Triple(triple)
								}
								treeldr_rust_prelude::rdf::TripleOrValue::Value(value) => {
									treeldr_rust_prelude::rdf::TripleOrValue::Triple(::rdf_types::Triple(
										self.id_.clone().unwrap(),
										treeldr_rust_prelude::rdf_types::FromIri::from_iri(
											vocabulary.insert(::treeldr_rust_prelude::static_iref::iri!(#prop_iri))
										),
										value
									))
								}
							})
						}
					},
					None => {
						quote! {
							.filer_map(|item| match item {
								treeldr_rust_prelude::rdf::TripleOrValue::Triple(triple) => Some(treeldr_rust_prelude::rdf::TripleOrValue::Triple(triple)),
								treeldr_rust_prelude::rdf::TripleOrValue::Value(value) => None
							})
						}
					}
				};

				next = quote! {
					self.#field_ident
						.next_with(
							vocabulary,
							generator
						)
						#map_prop_item
						.or_else(|| #next)
				};
			}
		}

		let iterator_id_init = iterator_id_init.unwrap_or_else(|| quote! {
			generator.next(namespace)
		});

		let bounds = bounds
			.separated_by(&quote!(,))
			.generate_with(context, scope)
			.into_tokens()?;

		tokens.extend(quote! {
			pub struct #iterator_ident<'a, I, V> {
				id_: Option<I>,
				#(#iterator_fields),*
			}

			impl<'a, N: ::treeldr_rust_prelude::rdf_types::Namespace, V> ::treeldr_rust_prelude::RdfIterator<N> for #iterator_ident<'a, N::Id, V>
			where
				N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
				N::Id: Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
				#bounds
			{
				type Item = ::treeldr_rust_prelude::rdf::TripleOrValue<N::Id, V>;

				fn next_with<
					G: ::treeldr_rust_prelude::rdf_types::Generator<N>
				>(
					&mut self,
					vocabulary: &mut N,
					generator: &mut G
				) -> Option<Self::Item> {
					#next
				}
			}

			impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V> ::treeldr_rust_prelude::rdf::TriplesAndValues<N, V> for #ident #params
			where
				N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
				N::Id: Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
				#bounds
			{
				type TriplesAndValues<'a> = #iterator_ident<'a, N::Id, V> where Self: 'a;

				fn unbound_rdf_triples_and_values<
					G: ::treeldr_rust_prelude::rdf_types::Generator<N>
				>(
					&self,
					namespace: &mut N,
					generator: &mut G
				) -> Self::TriplesAndValues<'_> {
					#iterator_ident {
						id_: Some(#iterator_id_init),
						#(#iterator_fields_init),*
					}
				}
			}
		});

		Ok(())
	}
}

impl<M> GenerateFor<Enum, M> for RdfTriplesImpl {
	fn generate<V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>>(
		&self,
		context: &crate::Context<V, M>,
		scope: Option<shelves::Ref<crate::Module>>,
		ty: &Enum,
		tokens: &mut proc_macro2::TokenStream,
	) -> Result<(), crate::Error> {
		todo!()
	}
}