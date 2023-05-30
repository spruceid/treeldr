mod real;

use std::fmt;

use rdf_types::{IriVocabulary, RdfDisplayWithContext};
pub use real::*;
use xsd_types::{
	Byte, Double, Float, Int, Integer, Long, NegativeInteger, NonNegativeInteger,
	NonPositiveInteger, PositiveInteger, Short, UnsignedByte, UnsignedInt, UnsignedLong,
	UnsignedShort,
};

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

	pub fn into_non_positive_integer(self) -> Result<NonPositiveInteger, Self> {
		match self {
			Self::Real(r) => r.into_non_positive_integer().map_err(Self::Real),
			other => Err(other),
		}
	}

	pub fn into_positive_integer(self) -> Result<PositiveInteger, Self> {
		match self {
			Self::Real(r) => r.into_positive_integer().map_err(Self::Real),
			other => Err(other),
		}
	}

	pub fn into_negative_integer(self) -> Result<NegativeInteger, Self> {
		match self {
			Self::Real(r) => r.into_negative_integer().map_err(Self::Real),
			other => Err(other),
		}
	}

	pub fn into_unsigned_long(self) -> Result<UnsignedLong, Self> {
		match self {
			Self::Real(r) => r.into_unsigned_long().map_err(Self::Real),
			other => Err(other),
		}
	}

	pub fn into_unsigned_int(self) -> Result<UnsignedInt, Self> {
		match self {
			Self::Real(r) => r.into_unsigned_int().map_err(Self::Real),
			other => Err(other),
		}
	}

	pub fn into_unsigned_short(self) -> Result<UnsignedShort, Self> {
		match self {
			Self::Real(r) => r.into_unsigned_short().map_err(Self::Real),
			other => Err(other),
		}
	}

	pub fn into_unsigned_byte(self) -> Result<UnsignedByte, Self> {
		match self {
			Self::Real(r) => r.into_unsigned_byte().map_err(Self::Real),
			other => Err(other),
		}
	}

	pub fn into_long(self) -> Result<Long, Self> {
		match self {
			Self::Real(r) => r.into_long().map_err(Self::Real),
			other => Err(other),
		}
	}

	pub fn into_int(self) -> Result<Int, Self> {
		match self {
			Self::Real(r) => r.into_int().map_err(Self::Real),
			other => Err(other),
		}
	}

	pub fn into_short(self) -> Result<Short, Self> {
		match self {
			Self::Real(r) => r.into_short().map_err(Self::Real),
			other => Err(other),
		}
	}

	pub fn into_byte(self) -> Result<Byte, Self> {
		match self {
			Self::Real(r) => r.into_byte().map_err(Self::Real),
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

impl From<NonPositiveInteger> for Numeric {
	fn from(value: NonPositiveInteger) -> Self {
		Self::Real(value.into())
	}
}

impl From<PositiveInteger> for Numeric {
	fn from(value: PositiveInteger) -> Self {
		Self::Real(value.into())
	}
}

impl From<NegativeInteger> for Numeric {
	fn from(value: NegativeInteger) -> Self {
		Self::Real(value.into())
	}
}

impl From<Long> for Numeric {
	fn from(value: Long) -> Self {
		Self::Real(value.into())
	}
}

impl From<Int> for Numeric {
	fn from(value: Int) -> Self {
		Self::Real(value.into())
	}
}

impl From<Short> for Numeric {
	fn from(value: Short) -> Self {
		Self::Real(value.into())
	}
}

impl From<Byte> for Numeric {
	fn from(value: Byte) -> Self {
		Self::Real(value.into())
	}
}

impl From<UnsignedLong> for Numeric {
	fn from(value: UnsignedLong) -> Self {
		Self::Real(value.into())
	}
}

impl From<UnsignedInt> for Numeric {
	fn from(value: UnsignedInt) -> Self {
		Self::Real(value.into())
	}
}

impl From<UnsignedShort> for Numeric {
	fn from(value: UnsignedShort) -> Self {
		Self::Real(value.into())
	}
}

impl From<UnsignedByte> for Numeric {
	fn from(value: UnsignedByte) -> Self {
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

impl TryFrom<Numeric> for Float {
	type Error = Numeric;

	fn try_from(value: Numeric) -> Result<Self, Self::Error> {
		match value {
			Numeric::Float(f) => Ok(f),
			n => Err(n),
		}
	}
}

impl TryFrom<Numeric> for Double {
	type Error = Numeric;

	fn try_from(value: Numeric) -> Result<Self, Self::Error> {
		match value {
			Numeric::Double(f) => Ok(f),
			n => Err(n),
		}
	}
}

impl TryFrom<Numeric> for Integer {
	type Error = Numeric;

	fn try_from(value: Numeric) -> Result<Self, Self::Error> {
		match value {
			Numeric::Real(Real::Rational(r)) => r
				.into_integer()
				.map_err(|r| Numeric::Real(Real::Rational(r))),
			n => Err(n),
		}
	}
}

impl TryFrom<Numeric> for NonNegativeInteger {
	type Error = Numeric;

	fn try_from(value: Numeric) -> Result<Self, Self::Error> {
		match value {
			Numeric::Real(Real::Rational(r)) => r
				.into_non_negative_integer()
				.map_err(|r| Numeric::Real(Real::Rational(r))),
			n => Err(n),
		}
	}
}

impl TryFrom<Numeric> for NonPositiveInteger {
	type Error = Numeric;

	fn try_from(value: Numeric) -> Result<Self, Self::Error> {
		match value {
			Numeric::Real(Real::Rational(r)) => r
				.into_non_positive_integer()
				.map_err(|r| Numeric::Real(Real::Rational(r))),
			n => Err(n),
		}
	}
}

impl TryFrom<Numeric> for PositiveInteger {
	type Error = Numeric;

	fn try_from(value: Numeric) -> Result<Self, Self::Error> {
		match value {
			Numeric::Real(Real::Rational(r)) => r
				.into_positive_integer()
				.map_err(|r| Numeric::Real(Real::Rational(r))),
			n => Err(n),
		}
	}
}

impl TryFrom<Numeric> for NegativeInteger {
	type Error = Numeric;

	fn try_from(value: Numeric) -> Result<Self, Self::Error> {
		match value {
			Numeric::Real(Real::Rational(r)) => r
				.into_negative_integer()
				.map_err(|r| Numeric::Real(Real::Rational(r))),
			n => Err(n),
		}
	}
}

impl TryFrom<Numeric> for Long {
	type Error = Numeric;

	fn try_from(value: Numeric) -> Result<Self, Self::Error> {
		match value {
			Numeric::Real(Real::Rational(r)) => {
				r.into_long().map_err(|r| Numeric::Real(Real::Rational(r)))
			}
			n => Err(n),
		}
	}
}

impl TryFrom<Numeric> for Int {
	type Error = Numeric;

	fn try_from(value: Numeric) -> Result<Self, Self::Error> {
		match value {
			Numeric::Real(Real::Rational(r)) => {
				r.into_int().map_err(|r| Numeric::Real(Real::Rational(r)))
			}
			n => Err(n),
		}
	}
}

impl TryFrom<Numeric> for Short {
	type Error = Numeric;

	fn try_from(value: Numeric) -> Result<Self, Self::Error> {
		match value {
			Numeric::Real(Real::Rational(r)) => {
				r.into_short().map_err(|r| Numeric::Real(Real::Rational(r)))
			}
			n => Err(n),
		}
	}
}

impl TryFrom<Numeric> for Byte {
	type Error = Numeric;

	fn try_from(value: Numeric) -> Result<Self, Self::Error> {
		match value {
			Numeric::Real(Real::Rational(r)) => {
				r.into_byte().map_err(|r| Numeric::Real(Real::Rational(r)))
			}
			n => Err(n),
		}
	}
}

impl TryFrom<Numeric> for UnsignedLong {
	type Error = Numeric;

	fn try_from(value: Numeric) -> Result<Self, Self::Error> {
		match value {
			Numeric::Real(Real::Rational(r)) => r
				.into_unsigned_long()
				.map_err(|r| Numeric::Real(Real::Rational(r))),
			n => Err(n),
		}
	}
}

impl TryFrom<Numeric> for UnsignedInt {
	type Error = Numeric;

	fn try_from(value: Numeric) -> Result<Self, Self::Error> {
		match value {
			Numeric::Real(Real::Rational(r)) => r
				.into_unsigned_int()
				.map_err(|r| Numeric::Real(Real::Rational(r))),
			n => Err(n),
		}
	}
}

impl TryFrom<Numeric> for UnsignedShort {
	type Error = Numeric;

	fn try_from(value: Numeric) -> Result<Self, Self::Error> {
		match value {
			Numeric::Real(Real::Rational(r)) => r
				.into_unsigned_short()
				.map_err(|r| Numeric::Real(Real::Rational(r))),
			n => Err(n),
		}
	}
}

impl TryFrom<Numeric> for UnsignedByte {
	type Error = Numeric;

	fn try_from(value: Numeric) -> Result<Self, Self::Error> {
		match value {
			Numeric::Real(Real::Rational(r)) => r
				.into_unsigned_byte()
				.map_err(|r| Numeric::Real(Real::Rational(r))),
			n => Err(n),
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum NumericRef<'a> {
	Real(RealRef<'a>),
	Float(Float),
	Double(Double),
}