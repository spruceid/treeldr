mod real;

use std::fmt;

use rdf_types::{IriVocabulary, RdfDisplayWithContext};
pub use real::*;
use xsd_types::{Double, Float, Integer, NonNegativeInteger};

use crate::IriIndex;

use super::AsRdfLiteral;

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Numeric {
	Real(Real),
	Float(Float),
	Double(Double),
}

impl Numeric {
	pub fn into_integer(self) -> Result<Integer, Self> {
		match self {
			Self::Real(r) => r.into_integer().map_err(Self::Real),
			other => Err(other),
		}
	}

	pub fn into_non_negative_integer(self) -> Result<NonNegativeInteger, Self> {
		match self {
			Self::Real(r) => r.into_non_negative_integer().map_err(Self::Real),
			other => Err(other),
		}
	}

	pub fn into_float(self) -> Result<Float, Self> {
		match self {
			Self::Float(f) => Ok(f),
			other => Err(other),
		}
	}

	pub fn into_double(self) -> Result<Double, Self> {
		match self {
			Self::Double(d) => Ok(d),
			other => Err(other),
		}
	}
}

impl<V: IriVocabulary<Iri = IriIndex>> RdfDisplayWithContext<V> for Numeric {
	fn rdf_fmt_with(&self, vocabulary: &V, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Real(v) => write!(f, "{v}^^{}", vocabulary.iri(&self.rdf_type()).unwrap()),
			Self::Double(v) => write!(f, "{v}^^{}", vocabulary.iri(&self.rdf_type()).unwrap()),
			Self::Float(v) => write!(f, "{v}^^{}", vocabulary.iri(&self.rdf_type()).unwrap()),
		}
	}
}

impl fmt::Display for Numeric {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Real(r) => r.fmt(f),
			Self::Float(d) => d.fmt(f),
			Self::Double(d) => d.fmt(f),
		}
	}
}

impl From<Real> for Numeric {
	fn from(value: Real) -> Self {
		Self::Real(value)
	}
}

impl From<Integer> for Numeric {
	fn from(value: Integer) -> Self {
		Self::Real(value.into())
	}
}

impl From<NonNegativeInteger> for Numeric {
	fn from(value: NonNegativeInteger) -> Self {
		Self::Real(value.into())
	}
}

impl From<Float> for Numeric {
	fn from(value: Float) -> Self {
		Self::Float(value)
	}
}

impl From<Double> for Numeric {
	fn from(value: Double) -> Self {
		Self::Double(value)
	}
}
