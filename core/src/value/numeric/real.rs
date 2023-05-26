use std::fmt;

use crate::vocab::StrippedLiteral;

mod rational;

use num::BigInt;
pub use rational::*;
use xsd_types::{
	Byte, Decimal, Int, Integer, Long, NegativeInteger, NonNegativeInteger, NonPositiveInteger,
	PositiveInteger, Short, UnsignedByte, UnsignedInt, UnsignedLong, UnsignedShort,
};

/// Real number value.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Real {
	Rational(Rational),
}

impl Real {
	pub fn into_integer(self) -> Result<Integer, Self> {
		match self {
			Self::Rational(r) => r.into_integer().map_err(Self::Rational),
		}
	}

	pub fn into_non_negative_integer(self) -> Result<NonNegativeInteger, Self> {
		match self {
			Self::Rational(r) => r.into_non_negative_integer().map_err(Self::Rational),
		}
	}

	pub fn literal(&self) -> StrippedLiteral {
		match self {
			Self::Rational(r) => r.literal(),
		}
	}
}

impl fmt::Display for Real {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Rational(r) => r.fmt(f),
		}
	}
}

impl From<super::Rational> for Real {
	fn from(value: super::Rational) -> Self {
		Self::Rational(value)
	}
}

impl From<Decimal> for Real {
	fn from(value: Decimal) -> Self {
		Self::Rational(value.into())
	}
}

impl From<Integer> for Real {
	fn from(value: Integer) -> Self {
		let n: BigInt = value.into();
		Self::Rational(n.into())
	}
}

impl From<NonNegativeInteger> for Real {
	fn from(value: NonNegativeInteger) -> Self {
		let n: BigInt = value.into();
		Self::Rational(n.into())
	}
}

impl From<NonPositiveInteger> for Real {
	fn from(value: NonPositiveInteger) -> Self {
		let n: BigInt = value.into();
		Self::Rational(n.into())
	}
}

impl From<PositiveInteger> for Real {
	fn from(value: PositiveInteger) -> Self {
		let n: BigInt = value.into_big_int();
		Self::Rational(n.into())
	}
}

impl From<NegativeInteger> for Real {
	fn from(value: NegativeInteger) -> Self {
		let n: BigInt = value.into_big_int();
		Self::Rational(n.into())
	}
}

impl From<Long> for Real {
	fn from(value: Long) -> Self {
		let n: BigInt = value.into();
		Self::Rational(n.into())
	}
}

impl From<Int> for Real {
	fn from(value: Int) -> Self {
		let n: BigInt = value.into();
		Self::Rational(n.into())
	}
}

impl From<Short> for Real {
	fn from(value: Short) -> Self {
		let n: BigInt = value.into();
		Self::Rational(n.into())
	}
}

impl From<Byte> for Real {
	fn from(value: Byte) -> Self {
		let n: BigInt = value.into();
		Self::Rational(n.into())
	}
}

impl From<UnsignedLong> for Real {
	fn from(value: UnsignedLong) -> Self {
		let n: BigInt = value.into();
		Self::Rational(n.into())
	}
}

impl From<UnsignedInt> for Real {
	fn from(value: UnsignedInt) -> Self {
		let n: BigInt = value.into();
		Self::Rational(n.into())
	}
}

impl From<UnsignedShort> for Real {
	fn from(value: UnsignedShort) -> Self {
		let n: BigInt = value.into();
		Self::Rational(n.into())
	}
}

impl From<UnsignedByte> for Real {
	fn from(value: UnsignedByte) -> Self {
		let n: BigInt = value.into();
		Self::Rational(n.into())
	}
}

impl TryFrom<Real> for Rational {
	type Error = Real;

	fn try_from(r: Real) -> Result<Self, Self::Error> {
		match r {
			Real::Rational(r) => Ok(r),
		}
	}
}

impl TryFrom<Real> for Decimal {
	type Error = Real;

	fn try_from(r: Real) -> Result<Self, Self::Error> {
		match r {
			Real::Rational(r) => r.try_into().map_err(Real::Rational),
		}
	}
}

impl TryFrom<Real> for Integer {
	type Error = Real;

	fn try_from(r: Real) -> Result<Self, Self::Error> {
		match r {
			Real::Rational(r) => r.try_into().map_err(Real::Rational),
		}
	}
}

impl TryFrom<Real> for NonNegativeInteger {
	type Error = Real;

	fn try_from(r: Real) -> Result<Self, Self::Error> {
		match r {
			Real::Rational(r) => r.try_into().map_err(Real::Rational),
		}
	}
}

/// Real number value.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum RealRef<'a> {
	Rational(&'a Rational),
}