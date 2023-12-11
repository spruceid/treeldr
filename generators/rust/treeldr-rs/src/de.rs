mod matching;

pub use matching::Matching;
use rdf_types::{ReverseTermInterpretation, TermInterpretation, Vocabulary};

use crate::{pattern::Substitution, Pattern, RdfContext, RdfType};

pub fn select_inputs<R: Clone, const N: usize>(
	inputs: &[Pattern<R>; N],
	substitution: &Substitution<R>,
) -> [R; N] {
	inputs
		.iter()
		.map(|p| p.apply(substitution).into_resource().unwrap())
		.collect::<Vec<_>>()
		.try_into()
		.ok()
		.unwrap()
}

pub fn select_graph<R: Clone>(
	current_graph: Option<&R>,
	graph_pattern: &Option<Option<Pattern<R>>>,
	substitution: &Substitution<R>,
) -> Option<R> {
	graph_pattern
		.as_ref()
		.map(|g| {
			g.as_ref()
				.map(|p| p.apply(substitution).into_resource().unwrap())
		})
		.unwrap_or_else(|| current_graph.cloned())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("data ambiguity")]
	DataAmbiguity,

	#[error("missing required data")]
	MissingData,

	#[error("missing required field `{0}`")]
	MissingField(String),

	#[error("missing required resource identifier")]
	MissingId,

	#[error("ambiguous resource identifier")]
	AmbiguousId,

	#[error("invalid resource identifier")]
	InvalidId,

	#[error("ambiguous literal value")]
	AmbiguousLiteralValue,

	#[error("invalid literal value")]
	InvalidLiteralValue,

	#[error("literal type mismatch")]
	LiteralTypeMismatch,

	#[error("expected literal value")]
	ExpectedLiteral,
}

impl From<matching::Error> for Error {
	fn from(value: matching::Error) -> Self {
		match value {
			matching::Error::Ambiguity => Self::DataAmbiguity,
			matching::Error::Empty => Self::MissingData,
		}
	}
}

pub trait DeserializeLd<const N: usize, V, I>: Sized
where
	V: Vocabulary<Value = String, Type = RdfType<V>>,
	I: TermInterpretation<V::Iri, V::BlankId, V::Literal>
		+ ReverseTermInterpretation<Iri = V::Iri, BlankId = V::BlankId, Literal = V::Literal>,
	I::Resource: Clone + Ord,
{
	fn deserialize_ld_with<D>(
		rdf: RdfContext<V, I>,
		dataset: &D,
		graph: Option<&I::Resource>,
		inputs: &[I::Resource; N],
	) -> Result<Self, Error>
	where
		D: grdf::Dataset<
			Subject = I::Resource,
			Predicate = I::Resource,
			Object = I::Resource,
			GraphLabel = I::Resource,
		>;
}

pub struct InvalidLiteral<T = String>(pub T);

pub trait FromRdfLiteral: Sized {
	fn from_rdf_literal(value: &str) -> Result<Self, InvalidLiteral>;
}

impl FromRdfLiteral for bool {
	fn from_rdf_literal(value: &str) -> Result<Self, InvalidLiteral> {
		use xsd_types::ParseRdf;
		let value =
			xsd_types::Boolean::parse_rdf(value).map_err(|_| InvalidLiteral(value.to_owned()))?;
		Ok(value.0)
	}
}

macro_rules! xsd_from_rdf {
	($($ty:ident),*) => {
		$(
			impl FromRdfLiteral for $ty {
				fn from_rdf_literal(value: &str) -> Result<Self, InvalidLiteral> {
					xsd_types::ParseRdf::parse_rdf(value).map_err(|_| InvalidLiteral(value.to_owned()))
				}
			}
		)*
	};
}

xsd_from_rdf!(u8, u16, u32, u64, i8, i16, i32, i64, String);
