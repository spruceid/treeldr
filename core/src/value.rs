use std::{borrow::Cow, fmt};

use langtag::LanguageTag;
use locspan::Meta;
use rdf_types::IriVocabulary;
pub use xsd_types::value::*;

use crate::{vocab, Id, IriIndex};

mod lang_string;
mod numeric;

pub use lang_string::LangString;
pub use numeric::*;

/// Value.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value {
	Node(Id),
	Literal(Literal),
}

impl Value {
	pub fn as_id(&self) -> Option<Id> {
		match self {
			Self::Node(id) => Some(*id),
			Self::Literal(_) => None,
		}
	}
}

impl<M> TryFrom<vocab::Object<M>> for Value {
	type Error = InvalidLiteral;

	fn try_from(value: vocab::Object<M>) -> Result<Self, Self::Error> {
		match value {
			vocab::Object::Literal(l) => Ok(Value::Literal(l.try_into()?)),
			vocab::Object::Id(id) => Ok(Value::Node(id)),
		}
	}
}

/// Literal value.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Literal {
	Numeric(Numeric),
	LangString(LangString),
	String(String),
	Other(String, IriIndex),
}

impl Literal {
	pub fn lexical_form(&self) -> Cow<str> {
		match self {
			Self::Numeric(n) => Cow::Owned(n.to_string()),
			Self::LangString(s) => Cow::Borrowed(s.as_str()),
			Self::String(s) => Cow::Borrowed(s.as_str()),
			Self::Other(s, _) => Cow::Borrowed(s.as_str()),
		}
	}
}

impl From<String> for Literal {
	fn from(value: String) -> Self {
		Self::String(value)
	}
}

impl fmt::Display for Literal {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Numeric(n) => n.fmt(f),
			Self::LangString(s) => s.fmt(f),
			Self::String(s) => s.fmt(f),
			Self::Other(s, _) => s.fmt(f),
		}
	}
}

impl<V: IriVocabulary<Iri = IriIndex>> rdf_types::RdfDisplayWithContext<V> for Literal {
	fn rdf_fmt_with(&self, vocabulary: &V, f: &mut fmt::Formatter) -> fmt::Result {
		use fmt::Display;
		use rdf_types::RdfDisplay;
		match self {
			Self::Numeric(n) => n.rdf_fmt_with(vocabulary, f),
			Self::LangString(s) => s.rdf_fmt(f),
			Self::String(s) => s.fmt(f),
			Self::Other(s, ty) => write!(f, "{s}^^{}", vocabulary.iri(ty).unwrap()),
		}
	}
}

#[derive(Debug, thiserror::Error)]
pub enum InvalidLiteral {
	#[error("missing language tag")]
	MissingLanguageTag,
}

impl<M> TryFrom<vocab::Literal<M>> for Literal {
	type Error = InvalidLiteral;

	fn try_from(value: vocab::Literal<M>) -> Result<Self, Self::Error> {
		match value {
			vocab::Literal::String(s) => Ok(Literal::String(s.into_value())),
			vocab::Literal::TypedString(s, Meta(ty, _)) => match ty {
				IriIndex::Iri(vocab::Term::Xsd(vocab::Xsd::String)) => {
					Ok(Literal::String(s.into_value()))
				}
				IriIndex::Iri(vocab::Term::Rdf(vocab::Rdf::LangString)) => {
					Err(InvalidLiteral::MissingLanguageTag)
				}
				ty => Ok(Literal::Other(s.into_value(), ty)),
			},
			vocab::Literal::LangString(s, tag) => Ok(Literal::LangString(LangString::new(
				s.into_value(),
				tag.into_value(),
			))),
		}
	}
}

pub trait AsRdfLiteral: Sized + fmt::Display {
	fn rdf_type(&self) -> IriIndex;

	fn language(&self) -> Option<LanguageTag> {
		None
	}

	fn as_rdf_literal(&self) -> rdf_types::Literal<String, IriIndex> {
		match self.rdf_type() {
			IriIndex::Iri(crate::vocab::Term::Xsd(crate::vocab::Xsd::String)) => {
				rdf_types::Literal::String(self.to_string())
			}
			IriIndex::Iri(crate::vocab::Term::Rdf(crate::vocab::Rdf::LangString)) => {
				rdf_types::Literal::LangString(self.to_string(), self.language().unwrap().cloned())
			}
			ty => rdf_types::Literal::TypedString(self.to_string(), ty),
		}
	}
}

impl AsRdfLiteral for Real {
	fn rdf_type(&self) -> IriIndex {
		IriIndex::Iri(crate::vocab::Term::Owl(crate::vocab::Owl::Real))
	}
}

impl AsRdfLiteral for Rational {
	fn rdf_type(&self) -> IriIndex {
		IriIndex::Iri(crate::vocab::Term::Owl(crate::vocab::Owl::Real))
	}
}

impl AsRdfLiteral for Float {
	fn rdf_type(&self) -> IriIndex {
		IriIndex::Iri(crate::vocab::Term::Xsd(crate::vocab::Xsd::Float))
	}
}

impl AsRdfLiteral for Double {
	fn rdf_type(&self) -> IriIndex {
		IriIndex::Iri(crate::vocab::Term::Xsd(crate::vocab::Xsd::Double))
	}
}

impl AsRdfLiteral for Decimal {
	fn rdf_type(&self) -> IriIndex {
		IriIndex::Iri(crate::vocab::Term::Xsd(crate::vocab::Xsd::Decimal))
	}
}

impl AsRdfLiteral for Integer {
	fn rdf_type(&self) -> IriIndex {
		IriIndex::Iri(crate::vocab::Term::Xsd(crate::vocab::Xsd::Integer))
	}
}

impl AsRdfLiteral for NonNegativeInteger {
	fn rdf_type(&self) -> IriIndex {
		IriIndex::Iri(crate::vocab::Term::Xsd(
			crate::vocab::Xsd::NonNegativeInteger,
		))
	}
}

// impl AsRdfLiteral for PositiveInteger {
// 	fn rdf_type(&self) -> IriIndex {
// 		IriIndex::Iri(crate::vocab::Term::Xsd(crate::vocab::Xsd::PositiveInteger))
// 	}
// }

impl AsRdfLiteral for NonPositiveInteger {
	fn rdf_type(&self) -> IriIndex {
		IriIndex::Iri(crate::vocab::Term::Xsd(
			crate::vocab::Xsd::NonPositiveInteger,
		))
	}
}

// impl AsRdfLiteral for NegativeInteger {
// 	fn rdf_type(&self) -> IriIndex {
// 		IriIndex::Iri(crate::vocab::Term::Xsd(crate::vocab::Xsd::NegativeInteger))
// 	}
// }

impl AsRdfLiteral for Numeric {
	fn rdf_type(&self) -> IriIndex {
		match self {
			Self::Real(r) => r.rdf_type(),
			Self::Double(d) => d.rdf_type(),
			Self::Float(f) => f.rdf_type(),
		}
	}
}

impl AsRdfLiteral for LangString {
	fn rdf_type(&self) -> IriIndex {
		IriIndex::Iri(crate::vocab::Term::Rdf(crate::vocab::Rdf::LangString))
	}

	fn language(&self) -> Option<LanguageTag> {
		Some(self.language())
	}
}

impl AsRdfLiteral for String {
	fn rdf_type(&self) -> IriIndex {
		IriIndex::Iri(crate::vocab::Term::Xsd(crate::vocab::Xsd::String))
	}
}

impl AsRdfLiteral for Literal {
	fn rdf_type(&self) -> IriIndex {
		match self {
			Self::Numeric(n) => n.rdf_type(),
			Self::LangString(s) => s.rdf_type(),
			Self::String(s) => s.rdf_type(),
			Self::Other(_, ty) => *ty,
		}
	}
}
