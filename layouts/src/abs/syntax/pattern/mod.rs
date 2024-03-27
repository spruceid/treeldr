mod compact_iri;
mod literal;
mod variable;

use core::fmt;

pub use compact_iri::*;
use iref::IriRefBuf;
pub use literal::*;
use rdf_types::{BlankIdBuf, Id, Term, RDF_NIL};
use serde::{Deserialize, Serialize};
pub use variable::*;

use super::{Error, Scope};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Pattern {
	Var(VariableNameBuf),
	Iri(CompactIri),
	Literal(LiteralValue),
}

impl Pattern {
	pub fn is_variable(&self, name: &str) -> bool {
		match self {
			Self::Var(x) => x == name,
			_ => false,
		}
	}

	pub fn default_head() -> Self {
		Self::Var(VariableNameBuf("self".to_string()))
	}

	pub fn is_default_head(&self) -> bool {
		match self {
			Self::Var(x) => x == "self",
			_ => false,
		}
	}

	pub fn default_tail() -> Self {
		Self::Iri(CompactIri(RDF_NIL.to_owned().into()))
	}

	pub fn is_default_tail(&self) -> bool {
		match self {
			Self::Iri(CompactIri(iri_ref)) => iri_ref == RDF_NIL,
			_ => false,
		}
	}

	pub fn from_term(term: Term) -> Self {
		match term {
			Term::Id(Id::Iri(iri)) => Self::Iri(iri.into()),
			Term::Id(Id::Blank(b)) => Self::Var(VariableNameBuf(b.suffix().to_owned())),
			Term::Literal(l) => Self::Literal(l.into()),
		}
	}

	pub fn to_term(&self, scope: &Scope) -> Result<Term, Error> {
		match self {
			Self::Var(name) => Ok(Term::blank(BlankIdBuf::from_suffix(name).unwrap())),
			Self::Iri(compact_iri) => compact_iri.resolve(scope).map(Term::iri),
			Self::Literal(l) => Ok(Term::Literal(rdf_types::Literal::new(
				l.value.clone(),
				l.type_.resolve(scope)?,
			))),
		}
	}
}

impl Serialize for Pattern {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		match self {
			Self::Var(name) => format!("_:{name}").serialize(serializer),
			Self::Iri(compact_iri) => compact_iri.serialize(serializer),
			Self::Literal(l) => l.serialize(serializer),
		}
	}
}

impl<'de> Deserialize<'de> for Pattern {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		use serde::de::Error;

		#[derive(Serialize, Deserialize)]
		#[serde(untagged)]
		pub enum StringOrLiteral {
			String(String),
			Literal(LiteralValue),
		}

		struct Expected;

		impl serde::de::Expected for Expected {
			fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				write!(formatter, "an IRI, blank node identifier or literal value")
			}
		}

		match StringOrLiteral::deserialize(deserializer)? {
			StringOrLiteral::String(v) => match BlankIdBuf::new(v) {
				Ok(blank_id) => Ok(Pattern::Var(VariableNameBuf(blank_id.suffix().to_owned()))),
				Err(e) => match IriRefBuf::new(e.0) {
					Ok(iri_ref) => Ok(Pattern::Iri(CompactIri(iri_ref))),
					Err(e) => Err(D::Error::invalid_value(
						serde::de::Unexpected::Str(&e.0),
						&Expected,
					)),
				},
			},
			StringOrLiteral::Literal(l) => Ok(Self::Literal(l)),
		}
	}
}
