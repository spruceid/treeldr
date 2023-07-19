use iref::Iri;
use rdf_types::{Interpretation, IriVocabulary, LiteralVocabulary};
pub use rdf_types::{Literal, Object, Subject};

mod literal;

pub use literal::*;

#[derive(Clone, Copy, Debug)]
pub enum FromRdfError {
	Never,
	UnexpectedLiteralValue,
	ExpectedLiteralValue,
	UnexpectedType,
	InvalidLexicalRepresentation,
	MissingRequiredPropertyValue,
}

/// Import from an RDF graph.
pub trait FromRdf<V, I: Interpretation>: Sized {
	fn from_rdf<G>(
		vocabulary: &V,
		interpretation: &I,
		graph: &G,
		id: &I::Resource,
	) -> Result<Self, FromRdfError>
	where
		G: grdf::Graph<Subject = I::Resource, Predicate = I::Resource, Object = I::Resource>;
}

impl<V, I: Interpretation> FromRdf<V, I> for crate::Id<I::Resource>
where
	I::Resource: Clone,
{
	fn from_rdf<G>(
		_vocabulary: &V,
		_interpretation: &I,
		_graph: &G,
		id: &<I as Interpretation>::Resource,
	) -> Result<Self, FromRdfError>
	where
		G: grdf::Graph<
			Subject = <I as Interpretation>::Resource,
			Predicate = <I as Interpretation>::Resource,
			Object = <I as Interpretation>::Resource,
		>,
	{
		Ok(crate::Id(id.clone()))
	}
}

pub trait FromRdfLiteral<V: LiteralVocabulary>: Sized {
	fn from_rdf_literal_value(value: &V::Value) -> Result<Self, FromRdfError>;

	fn from_rdf_literal(
		vocabulary: &V,
		literal: &Literal<V::Type, V::Value>,
	) -> Result<Self, FromRdfError>;
}

/// Literal value type check.
pub trait TypeCheck<V> {
	fn has_type(&self, vocabulary: &V, iri: Iri) -> bool;
}

impl<V: IriVocabulary, S, L> TypeCheck<V> for Literal<rdf_types::literal::Type<V::Iri, L>, S> {
	fn has_type(&self, vocabulary: &V, iri: Iri) -> bool {
		match self.type_() {
			rdf_types::literal::Type::Any(t) => iri == vocabulary.iri(t).unwrap(),
			rdf_types::literal::Type::LangString(_) => {
				iri == static_iref::iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#langString")
			}
		}
	}
}
