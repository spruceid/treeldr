mod real;

pub use real::*;
use xsd_types::{Double, Float, Integer, NonNegativeInteger};

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
