use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

pub struct AsJsonLdImpl {
	pub type_path: syn::Type,
	pub body: TokenStream,
}

impl ToTokens for AsJsonLdImpl {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let type_path = &self.type_path;
		let body = &self.body;

		tokens.extend(quote! {
			impl<N: ::treeldr_rust_prelude::rdf_types::VocabularyMut> ::treeldr_rust_prelude::AsJsonLdObjectMeta<N> for #type_path
			where
				N: treeldr_rust_prelude::rdf_types::Namespace,
				N::Id: Clone + ::treeldr_rust_prelude::rdf_types::IntoId<Iri = N::Iri, BlankId = N::BlankId>,
				N::Iri: ::core::cmp::Eq + ::std::hash::Hash,
				N::BlankId: ::core::cmp::Eq + ::std::hash::Hash
			{
				fn as_json_ld_object_meta(
					&self,
					namespace: &mut N,
					meta: ()
				) -> ::treeldr_rust_prelude::json_ld::IndexedObject<N::Iri, N::BlankId, ()> {
					#body
				}
			}
		})
	}
}

pub struct IntoJsonLdImpl {
	pub type_path: syn::Type,
	pub body: TokenStream,
}

impl ToTokens for IntoJsonLdImpl {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let type_path = &self.type_path;
		let body = &self.body;

		tokens.extend(quote! {
			impl<N: ::treeldr_rust_prelude::rdf_types::VocabularyMut> ::treeldr_rust_prelude::IntoJsonLdObjectMeta<N> for #type_path
			where
				N: treeldr_rust_prelude::rdf_types::Namespace,
				N::Id: ::treeldr_rust_prelude::rdf_types::IntoId<Iri = N::Iri, BlankId = N::BlankId>,
				N::Iri: ::core::cmp::Eq + ::std::hash::Hash,
				N::BlankId: ::core::cmp::Eq + ::std::hash::Hash
			{
				fn into_json_ld_object_meta(
					self,
					namespace: &mut N,
					meta: ()
				) -> ::treeldr_rust_prelude::json_ld::IndexedObject<N::Iri, N::BlankId, ()> {
					#body
				}
			}
		})
	}
}

pub struct IntoJsonLdSyntaxImpl {
	pub type_path: syn::Type,
	pub body: TokenStream,
}

impl ToTokens for IntoJsonLdSyntaxImpl {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let type_path = &self.type_path;
		let body = &self.body;

		tokens.extend(quote! {
			impl<N: ::treeldr_rust_prelude::rdf_types::Namespace> ::treeldr_rust_prelude::IntoJsonLdSyntax<N> for #type_path where N::Id: ::treeldr_rust_prelude::contextual::DisplayWithContext<N> {
				fn into_json_ld_syntax(
					self,
					namespace: &N
				) -> ::treeldr_rust_prelude::json_ld::syntax::Value {
					#body
				}
			}
		})
	}
}
