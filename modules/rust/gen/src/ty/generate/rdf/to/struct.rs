use std::collections::BTreeSet;

use quote::quote;
use treeldr::Id;

use crate::{
	ty::{self, generate::GenerateFor, params::ParametersValues, structure::Struct},
	Generate, GenerateList,
};

use super::{
	collect_bounds, triples_and_values_iterator_name_from, triples_and_values_iterator_of,
	RdfTriplesImpl,
};

impl<M> GenerateFor<Struct, M> for RdfTriplesImpl {
	fn generate<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		context: &crate::Context<V, M>,
		scope: Option<shelves::Ref<crate::Module>>,
		ty: &Struct,
		tokens: &mut proc_macro2::TokenStream,
	) -> Result<(), crate::Error> {
		let ident = ty.ident();
		let def_params_values = ParametersValues::default();
		let impl_params_values = ParametersValues::new(quote!(N::Id));
		let params = ty.params().instantiate(&impl_params_values);
		let iterator_ident = triples_and_values_iterator_name_from(ident);

		let mut iterator_fields = Vec::with_capacity(ty.fields().len());
		let mut iterator_fields_init = Vec::with_capacity(ty.fields().len());
		let mut iterator_id_init = None;
		let mut next = quote!(self
			.id_
			.take()
			.map(::treeldr_rust_prelude::rdf_types::Object::Id)
			.map(::treeldr_rust_prelude::rdf::TripleOrValue::Value));
		let mut bounds = BTreeSet::new();
		for field in ty.fields() {
			let field_ident = field.ident();
			if field.is_self() {
				let ty = context.layout_type(field.layout()).unwrap();
				iterator_id_init = Some(match ty.description() {
					ty::Description::BuiltIn(ty::BuiltIn::Option(_)) => {
						quote! {
							self.#field_ident.clone().map(::treeldr_rust_prelude::Id::unwrap).unwrap_or_else(|| {
								generator.next(namespace)
							})
						}
					}
					ty::Description::BuiltIn(ty::BuiltIn::Required(_)) => {
						quote! {
							self.#field_ident.clone().unwrap()
						}
					}
					_ => panic!("invalid `tldr:self` layout"),
				})
			} else {
				let iter_ty = triples_and_values_iterator_of(
					context,
					scope,
					&def_params_values,
					field.layout(),
					quote!('a),
				)?;
				collect_bounds(context, field.layout(), |b| {
					bounds.insert(b);
				});
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
					}
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

		if iterator_fields.is_empty() {
			iterator_fields.push(quote! {
				_v: ::std::marker::PhantomData<&'a V>
			});

			iterator_fields_init.push(quote! {
				_v: ::std::marker::PhantomData
			})
		}

		let iterator_id_init = iterator_id_init.unwrap_or_else(|| {
			quote! {
				generator.next(namespace)
			}
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

			impl<'a, N: ::treeldr_rust_prelude::rdf_types::Namespace, V: 'a> ::treeldr_rust_prelude::RdfIterator<N> for #iterator_ident<'a, N::Id, V>
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
				type TriplesAndValues<'a> = #iterator_ident<'a, N::Id, V> where Self: 'a, N::Id: 'a, V: 'a;

				fn unbound_rdf_triples_and_values<
					'a,
					G: ::treeldr_rust_prelude::rdf_types::Generator<N>
				>(
					&'a self,
					namespace: &mut N,
					generator: &mut G
				) -> Self::TriplesAndValues<'a>
				where
					N::Id: 'a,
					V: 'a
				{
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
