use crate::{vocab::StrippedLiteral, IriIndex};
use num::{BigInt, Signed, Zero};
use std::fmt;

/// Integer number.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Integer(BigInt);

impl Integer {
	pub fn zero() -> Self {
		Self(num::BigInt::zero())
	}

	pub fn is_positive(&self) -> bool {
		self.0.is_positive()
	}

	pub fn is_negative(&self) -> bool {
		self.0.is_negative()
	}

	pub fn is_zero(&self) -> bool {
		self.0.is_zero()
	}

	pub fn into_i64(self) -> Result<i64, Self> {
		self.0
			.try_into()
			.map_err(|e: num::bigint::TryFromBigIntError<_>| Self(e.into_original()))
	}

	pub fn into_u64(self) -> Result<u64, Self> {
		self.0
			.try_into()
			.map_err(|e: num::bigint::TryFromBigIntError<_>| Self(e.into_original()))
	}

	pub fn literal(&self) -> StrippedLiteral {
		use crate::vocab::{Term, Xsd};
		StrippedLiteral::TypedString(
			self.0.to_string().into(),
			IriIndex::Iri(Term::Xsd(Xsd::Integer)),
		)
	}
}

impl fmt::Display for Integer {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(f)
	}
}

impl From<BigInt> for Integer {
	fn from(i: BigInt) -> Self {
		Self(i)
	}
}

impl From<xsd_types::IntegerBuf> for Integer {
	fn from(i: xsd_types::IntegerBuf) -> Self {
		BigInt::parse_bytes(i.as_bytes(), 10).unwrap().into()
	}
}

impl<'a> From<&'a xsd_types::Integer> for Integer {
	fn from(i: &'a xsd_types::Integer) -> Self {
		BigInt::parse_bytes(i.as_bytes(), 10).unwrap().into()
	}
}

impl From<Integer> for super::Real {
	fn from(i: Integer) -> Self {
		Self::Rational(i.into())
	}
}

impl From<Integer> for super::Rational {
	fn from(i: Integer) -> Self {
		i.0.into()
	}
}

impl From<Integer> for super::Decimal {
	fn from(i: Integer) -> Self {
		i.0.into()
	}
}

impl TryFrom<Integer> for super::NonNegativeInteger {
	type Error = Integer;

	fn try_from(i: Integer) -> Result<Self, Self::Error> {
		if !i.is_negative() {
			Ok(unsafe { Self::new_unchecked(i) })
		} else {
			Err(i)
		}
	}
}
