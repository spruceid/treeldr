use iref::Iri;
use rdf_types::{IriVocabulary, Namespace};
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
pub trait FromRdf<N: Namespace, L>: Sized {
	fn from_rdf<G>(
		namespace: &mut N,
		value: &Object<N::Id, L>,
		graph: &G,
	) -> Result<Self, FromRdfError>
	where
		G: grdf::Graph<Subject = N::Id, Predicate = N::Id, Object = Object<N::Id, L>>;
}

impl<N: Namespace, L> FromRdf<N, L> for crate::Id<N::Id>
where
	N::Id: Clone,
{
	fn from_rdf<G>(
		_namespace: &mut N,
		value: &Object<<N as Namespace>::Id, L>,
		_graph: &G,
	) -> Result<Self, FromRdfError>
	where
		G: grdf::Graph<
			Subject = <N as Namespace>::Id,
			Predicate = <N as Namespace>::Id,
			Object = Object<<N as Namespace>::Id, L>,
		>,
	{
		match value {
			Object::Id(id) => Ok(Self(id.clone())),
			_ => Err(FromRdfError::UnexpectedLiteralValue),
		}
	}
}

macro_rules! from_rdf_literal {
	($($ty:ty),*) => {
		$(
			impl<N: Namespace, L> FromRdf<N, L> for $ty where $ty: FromLiteral<L, N> {
				fn from_rdf<G>(namespace: &mut N, value: &Object<<N as Namespace>::Id, L>, _graph: &G) -> Result<Self, FromRdfError>
					where
						G: grdf::Graph<Subject = <N as Namespace>::Id, Predicate = <N as Namespace>::Id, Object = Object<<N as Namespace>::Id, L>> {
					match value {
						Object::Literal(l) => Self::from_literal(namespace, l),
						Object::Id(_) => Err(FromRdfError::ExpectedLiteralValue)
					}
				}
			}
		)*
	};
}

from_rdf_literal! {
	xsd_types::Boolean,
	xsd_types::Decimal,
	xsd_types::Integer,
	xsd_types::Long,
	xsd_types::Int,
	xsd_types::Short,
	xsd_types::Byte,
	xsd_types::NonNegativeInteger,
	xsd_types::PositiveInteger,
	xsd_types::UnsignedLong,
	xsd_types::UnsignedInt,
	xsd_types::UnsignedShort,
	xsd_types::UnsignedByte,
	xsd_types::NonPositiveInteger,
	xsd_types::NegativeInteger,
	xsd_types::Float,
	xsd_types::Double,
	xsd_types::String,
	xsd_types::Base64BinaryBuf,
	xsd_types::HexBinaryBuf,
	iref::IriBuf,
	::chrono::NaiveDate,
	::chrono::DateTime<::chrono::Utc>
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
