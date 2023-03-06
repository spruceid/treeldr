use std::collections::BTreeSet;

use quote::quote;

use crate::{
	ty::{enumeration::Enum, generate::GenerateFor, params::ParametersValues},
	Generate, GenerateList,
};

use super::{
	collect_bounds, triples_and_values_iterator_name_from, triples_and_values_iterator_of,
	RdfTriplesImpl,
};

impl<M> GenerateFor<Enum, M> for RdfTriplesImpl {
	fn generate<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		context: &crate::Context<V, M>,
		scope: Option<shelves::Ref<crate::Module>>,
		ty: &Enum,
		tokens: &mut proc_macro2::TokenStream,
	) -> Result<(), crate::Error> {
		let ident = ty.ident();
		let def_params_values = ParametersValues::default();
		let impl_params_values = ParametersValues::new(quote!(N::Id));

		let iterator_ident = triples_and_values_iterator_name_from(ident);
		let mut iterator_variants = Vec::with_capacity(ty.variants().len());
		let mut iterator_variants_next = Vec::with_capacity(ty.variants().len());
		let mut variants_init = Vec::with_capacity(ty.variants().len());
		let mut bounds = BTreeSet::new();
		for variant in ty.variants() {
			let variant_ident = variant.ident();
			match variant.ty() {
				Some(variant_type) => {
					let variant_iterator_type = triples_and_values_iterator_of(
						context,
						scope,
						&def_params_values,
						variant_type,
						quote!('a),
					)?;
					collect_bounds(context, variant_type, |b| {
						bounds.insert(b);
					});
					iterator_variants.push(quote! {
						#variant_ident ( #variant_iterator_type )
					});
					iterator_variants_next.push(quote! {
						Self::#variant_ident(inner) => inner.next_with(namespace, generator)
					});
					variants_init.push(quote! {
						Self::#variant_ident(value) => #iterator_ident::#variant_ident(value.unbound_rdf_triples_and_values(namespace, generator))
					})
				}
				None => {
					iterator_variants.push(quote! {
						#variant_ident
					});
					iterator_variants_next.push(quote! {
						Self::#variant_ident => None
					});
					variants_init.push(quote! {
						Self::#variant_ident => #iterator_ident::#variant_ident
					})
				}
			}
		}

		let params = ty.params().instantiate(&impl_params_values);

		let bounds = bounds
			.separated_by(&quote!(,))
			.generate_with(context, scope)
			.into_tokens()?;

		tokens.extend(quote! {
			pub enum #iterator_ident<'a, I, V> {
				#(#iterator_variants),*
			}

			impl<'a, N: ::treeldr_rust_prelude::rdf_types::Namespace, V: 'a> ::treeldr_rust_prelude::RdfIterator<N> for #iterator_ident<'a, N::Id, V>
			where
				N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
				N::Id: 'a + Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
				#bounds
			{
				type Item = ::treeldr_rust_prelude::rdf::TripleOrValue<N::Id, V>;

				fn next_with<
					G: ::treeldr_rust_prelude::rdf_types::Generator<N>
				>(
					&mut self,
					namespace: &mut N,
					generator: &mut G
				) -> Option<Self::Item> {
					match self {
						#(#iterator_variants_next),*
					}
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
					match self {
						#(#variants_init),*
					}
				}
			}
		});

		Ok(())
	}
}
