use proc_macro2::TokenStream;
use quote::quote;
use rdf_types::Vocabulary;
use treeldr::{BlankIdIndex, IriIndex};

use crate::{
	ty::{enumeration::Enum, generate::GenerateFor, params::ParametersValues},
	Context,
};

use super::FromRdfImpl;

impl<M> GenerateFor<Enum, M> for FromRdfImpl {
	fn generate<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		_context: &Context<V, M>,
		_scope: Option<shelves::Ref<crate::Module>>,
		ty: &Enum,
		tokens: &mut TokenStream,
	) -> Result<(), crate::Error> {
		let ident = ty.ident();
		let params_values = ParametersValues::new_for_type(quote!(N::Id));
		let params = ty.params().instantiate(&params_values);

		tokens.extend(quote! {
			impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V> ::treeldr_rust_prelude::FromRdf<N, V> for #ident #params
			where
				N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
				N::Id: Clone + Ord + ::treeldr_rust_prelude::rdf_types::FromIri<Iri=N::Iri>
			{
				fn from_rdf<G>(
					namespace: &mut N,
					id: &N::Id,
					graph: &G
				) -> Result<Self, ::treeldr_rust_prelude::FromRdfError>
				where
					G: ::treeldr_rust_prelude::grdf::Graph<
						Subject=N::Id,
						Predicate=N::Id,
						Object=::treeldr_rust_prelude::rdf_types::Object<N::Id, V>
					>
				{
					todo!()
				}
			}
		});

		Ok(())
	}
}
