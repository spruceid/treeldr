use iref::IriBuf;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

pub enum FromRdfImpl {
	Value(FromRdfValueImpl),
	Literal(FromRdfLiteralImpl),
}

impl ToTokens for FromRdfImpl {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		match self {
			Self::Value(i) => i.to_tokens(tokens),
			Self::Literal(i) => i.to_tokens(tokens),
		}
	}
}

pub struct FromRdfValueImpl {
	pub type_path: syn::Type,
	pub bounds: Vec<syn::WherePredicate>,
	pub from_id: TokenStream,
	pub from_literal: TokenStream,
}

impl ToTokens for FromRdfValueImpl {
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
				V: ::treeldr_rust_prelude::rdf::TypeCheck<N>,
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

pub struct FromRdfLiteralImpl {
	pub type_path: syn::Type,
	pub type_iri: Option<IriBuf>,
	pub base_type_path: syn::Type,
}

impl ToTokens for FromRdfLiteralImpl {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let type_path = &self.type_path;
		let base = &self.base_type_path;

		let body = match &self.type_iri {
			Some(ty_iri) => {
				let ty_iri = ty_iri.as_str();

				quote! {
					if ::treeldr_rust_prelude::rdf::TypeCheck::has_type(literal, namespace, ::treeldr_rust_prelude::static_iref::iri!(#ty_iri)) {
						::treeldr_rust_prelude::rdf::FromLiteral::<L, N>::from_literal_type_unchecked(literal)
					} else {
						Err(::treeldr_rust_prelude::FromRdfError::UnexpectedType)
					}
				}
			}
			None => {
				quote! {
					let base: #base = ::treeldr_rust_prelude::rdf::FromLiteral::<L, N>::from_literal(namespace, literal)?;

					match Self::try_from(base) {
						Ok(value) => Ok(value),
						Err(_) => Err(::treeldr_rust_prelude::FromRdfError::InvalidLexicalRepresentation)
					}
				}
			}
		};

		tokens.extend(quote! {
			impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, L> ::treeldr_rust_prelude::rdf::FromLiteral<L, N> for #type_path
			where
				N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
				N::Id: Clone + Ord + ::treeldr_rust_prelude::rdf_types::FromIri<Iri=N::Iri>,
				L: ::treeldr_rust_prelude::rdf::TypeCheck<N>,
				#base: ::treeldr_rust_prelude::rdf::FromLiteral<L, N>
			{
				fn from_literal_type_unchecked(literal: &L) -> Result<Self, ::treeldr_rust_prelude::FromRdfError> {
					let base: #base = ::treeldr_rust_prelude::rdf::FromLiteral::<L, N>::from_literal_type_unchecked(literal)?;

					match Self::try_from(base) {
						Ok(value) => Ok(value),
						Err(_) => Err(::treeldr_rust_prelude::FromRdfError::InvalidLexicalRepresentation)
					}
				}

				fn from_literal(namespace: &N, literal: &L) -> Result<Self, ::treeldr_rust_prelude::FromRdfError> {
					#body
				}
			}
		})
	}
}
