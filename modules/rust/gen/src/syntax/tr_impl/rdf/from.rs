use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

pub struct FromRdfImpl {
	pub type_path: syn::Type,
	pub bounds: Vec<syn::WherePredicate>,
	pub from_id: TokenStream,
	pub from_literal: TokenStream,
}

impl ToTokens for FromRdfImpl {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let type_path = &self.type_path;
		let bounds = &self.bounds;
		let from_id = &self.from_id;
		let from_literal = &self.from_literal;

		tokens.extend(quote! {
			impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V> ::treeldr_rust_prelude::FromRdf<N, V> for #type_path
			where
				N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
				N::Id: Clone + Ord + ::treeldr_rust_prelude::rdf_types::FromIri<Iri=N::Iri>,
				V: ::treeldr_rust_prelude::rdf::TypeCheck<N::Id>,
				#(#bounds),*
			{
				fn from_rdf<G>(
					namespace: &mut N,
					value: &::treeldr_rust_prelude::rdf_types::Object<N::Id, V>,
					graph: &G
				) -> Result<Self, ::treeldr_rust_prelude::FromRdfError>
				where
					G: ::treeldr_rust_prelude::grdf::Graph<
						Subject=N::Id,
						Predicate=N::Id,
						Object=::treeldr_rust_prelude::rdf_types::Object<N::Id, V>
					>
				{
					match value {
						::treeldr_rust_prelude::rdf_types::Object::Id(id) => {
							#from_id
						}
						::treeldr_rust_prelude::rdf_types::Object::Literal(literal) => {
							#from_literal
						}
					}
				}
			}
		})
	}
}
