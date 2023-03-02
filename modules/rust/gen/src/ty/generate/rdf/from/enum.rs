use proc_macro2::TokenStream;
use quote::quote;
use rdf_types::Vocabulary;
use treeldr::{IriIndex, BlankIdIndex};

use crate::{ty::{enumeration::Enum, generate::GenerateFor, params::ParametersValues}, Context};

use super::FromRdfImpl;

impl<M> GenerateFor<Enum, M>for FromRdfImpl {
	fn generate<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		_context: &Context<V, M>,
		_scope: Option<shelves::Ref<crate::Module>>,
		ty: &Enum,
		tokens: &mut TokenStream,
	) -> Result<(), crate::Error> {
		let ident = ty.ident();
		let params_values = ParametersValues::default();
		let params = ty.params().instantiate(&params_values);

		tokens.extend(quote! {
			impl<I, V, N> ::treeldr_rust_prelude::FromRdf<I, V, N> for #ident #params {
				fn from_rdf<G>(
					namespace: &N,
					id: &I,
					graph: &G
				) -> Result<Self, ::treeldr_rust_prelude::FromRdfError>
				where
					G: ::grdf::Graph<Subject=I, Predicate=I, Object=::treeldr_rust_prelude::rdf_types::Object<I, V>>
				{
					todo!()
				}
			}
		});

		Ok(())
	}
}