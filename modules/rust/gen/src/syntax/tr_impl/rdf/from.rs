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
	pub from_literal: Option<TokenStream>,
}

impl ToTokens for FromRdfValueImpl {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let type_path = &self.type_path;
		let bounds = &self.bounds;
		let from_id = &self.from_id;

		let body = match &self.from_literal {
			Some(from_literal) => {
				quote! {
					match interpretation.literals_of(id).next() {
						Some(literal) => {
							#from_literal
						}
						None => {
							#from_id
						}
					}
				}
			}
			None => {
				quote!(#from_id)
			}
		};

		tokens.extend(quote! {
			impl<V, I> ::treeldr_rust_prelude::FromRdf<V, I> for #type_path
			where
				V: ::treeldr_rust_prelude::rdf_types::Vocabulary,
				I: ::treeldr_rust_prelude::rdf_types::IriInterpretation<V::Iri> +
					::treeldr_rust_prelude::rdf_types::ReverseLiteralInterpretation<Literal = V::Literal>,
				I::Resource: Clone + Ord,
				#(#bounds),*
			{
				fn from_rdf<G>(
					vocabulary: &V,
					interpretation: &I,
					graph: &G,
					id: &I::Resource
				) -> Result<Self, ::treeldr_rust_prelude::FromRdfError>
				where
					G: ::treeldr_rust_prelude::grdf::Graph<
						Subject=I::Resource,
						Predicate=I::Resource,
						Object=I::Resource
					>
				{
					#body
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
					if ::treeldr_rust_prelude::rdf::TypeCheck::has_type(literal, vocabulary, ::treeldr_rust_prelude::static_iref::iri!(#ty_iri)) {
						::treeldr_rust_prelude::rdf::FromRdfLiteral::<V>::from_rdf_literal_value(literal.value())
					} else {
						Err(::treeldr_rust_prelude::FromRdfError::UnexpectedType)
					}
				}
			}
			None => {
				quote! {
					let base: #base = ::treeldr_rust_prelude::rdf::FromRdfLiteral::<V>::from_literal(vocabulary, literal)?;

					match Self::try_from(base) {
						Ok(value) => Ok(value),
						Err(_) => Err(::treeldr_rust_prelude::FromRdfError::InvalidLexicalRepresentation)
					}
				}
			}
		};

		tokens.extend(quote! {
			impl<V> ::treeldr_rust_prelude::rdf::FromRdfLiteral<V> for #type_path
			where
				V: ::treeldr_rust_prelude::rdf_types::Vocabulary<Type = ::treeldr_rust_prelude::rdf_types::literal::Type<<V as ::treeldr_rust_prelude::rdf_types::IriVocabulary>::Iri, <V as ::treeldr_rust_prelude::rdf_types::LanguageTagVocabulary>::LanguageTag>>,
				V::Value: AsRef<str>
			{
				fn from_rdf_literal_value(literal: &V::Value) -> Result<Self, ::treeldr_rust_prelude::FromRdfError> {
					let base: #base = ::treeldr_rust_prelude::rdf::FromRdfLiteral::<V>::from_rdf_literal_value(literal)?;

					match Self::try_from(base) {
						Ok(value) => Ok(value),
						Err(_) => Err(::treeldr_rust_prelude::rdf::FromRdfError::InvalidLexicalRepresentation)
					}
				}

				fn from_rdf_literal(
					vocabulary: &V,
					literal: &::treeldr_rust_prelude::rdf_types::Literal<V::Type, V::Value>
				) -> Result<Self, ::treeldr_rust_prelude::FromRdfError> {
					#body
				}
			}

			impl<V: ::treeldr_rust_prelude::rdf_types::LiteralVocabulary, I: ::treeldr_rust_prelude::rdf_types::ReverseLiteralInterpretation<Literal = V::Literal>> ::treeldr_rust_prelude::rdf::FromRdf<V, I> for #type_path
			where
				#type_path: ::treeldr_rust_prelude::rdf::FromRdfLiteral<V>
			{
				fn from_rdf<G>(
					vocabulary: &V,
					interpretation: &I,
					_graph: &G,
					id: &<I as ::treeldr_rust_prelude::rdf_types::Interpretation>::Resource
				) -> Result<Self, ::treeldr_rust_prelude::rdf::FromRdfError>
				where
					G: ::treeldr_rust_prelude::grdf::Graph<Subject = <I as ::treeldr_rust_prelude::rdf_types::Interpretation>::Resource, Predicate = <I as ::treeldr_rust_prelude::rdf_types::Interpretation>::Resource, Object = <I as ::treeldr_rust_prelude::rdf_types::Interpretation>::Resource>
				{
					let literal_id = interpretation.literals_of(id).next().ok_or(::treeldr_rust_prelude::rdf::FromRdfError::ExpectedLiteralValue)?;
					let literal = vocabulary.literal(literal_id).unwrap();
					<Self as ::treeldr_rust_prelude::rdf::FromRdfLiteral<V>>::from_rdf_literal(vocabulary, literal)
				}
			}
		})
	}
}
