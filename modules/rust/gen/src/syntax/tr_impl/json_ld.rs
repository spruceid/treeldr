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
			impl<V, I> ::treeldr_rust_prelude::AsJsonLdObjectMeta<V, I> for #type_path
			where
				V: ::treeldr_rust_prelude::rdf_types::VocabularyMut,
				V::Iri: Clone + Eq + ::std::hash::Hash,
				V::BlankId: Clone + Eq + ::std::hash::Hash,
				I: ::treeldr_rust_prelude::rdf_types::ReverseTermInterpretation<Iri = V::Iri, BlankId = V::BlankId, Literal = V::Literal>
			{
				fn as_json_ld_object_meta(
					&self,
					vocabulary: &mut V,
					interpretation: &I,
					meta: ()
				) -> ::treeldr_rust_prelude::json_ld::IndexedObject<V::Iri, V::BlankId, ()> {
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
			impl<V: ::treeldr_rust_prelude::rdf_types::VocabularyMut, I> ::treeldr_rust_prelude::IntoJsonLdObjectMeta<V, I> for #type_path
			where
			V: ::treeldr_rust_prelude::rdf_types::VocabularyMut,
			V::Iri: Clone + Eq + ::std::hash::Hash,
			V::BlankId: Clone + Eq + ::std::hash::Hash,
			I: ::treeldr_rust_prelude::rdf_types::ReverseTermInterpretation<Iri = V::Iri, BlankId = V::BlankId, Literal = V::Literal>
			{
				fn into_json_ld_object_meta(
					self,
					vocabulary: &mut V,
					interpretation: &I,
					meta: ()
				) -> ::treeldr_rust_prelude::json_ld::IndexedObject<V::Iri, V::BlankId, ()> {
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
			impl<V: ::treeldr_rust_prelude::rdf_types::Vocabulary, I: ::treeldr_rust_prelude::rdf_types::ReverseTermInterpretation<Iri = V::Iri, BlankId = V::BlankId, Literal = V::Literal>> ::treeldr_rust_prelude::IntoJsonLdSyntax<V, I> for #type_path
			where
				V::Value: ::std::convert::AsRef<str>
			{
				fn into_json_ld_syntax(
					self,
					vocabulary: &V,
					interpretation: &I
				) -> ::treeldr_rust_prelude::json_ld::syntax::Value {
					#body
				}
			}
		})
	}
}
