use super::Integer;
use crate::{vocab::StrippedLiteral, IriIndex};
use std::fmt;

/// Non Negative Integer number.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct NonNegativeInteger(Integer);

impl NonNegativeInteger {
	/// Create a new non negative integer from an integer, without checking
	/// that the input integer is indeed non negative.
	///
	/// ## Safety
	///
	/// The input integer must not be negative.
	pub unsafe fn new_unchecked(i: Integer) -> Self {
		Self(i)
	}

	pub fn zero() -> Self {
		Self(Integer::zero())
	}

	pub fn is_zero(&self) -> bool {
		self.0.is_zero()
	}

	pub fn into_u64(self) -> Result<u64, Self> {
		self.0.into_u64().map_err(Self)
	}

	pub fn literal(&self) -> StrippedLiteral {
		use crate::vocab::{Term, Xsd};
		StrippedLiteral::TypedString(
			self.0.to_string().into(),
			IriIndex::Iri(Term::Xsd(Xsd::NonNegativeInteger)),
		)
	}
}

impl fmt::Display for NonNegativeInteger {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(f)
	}
}

impl From<NonNegativeInteger> for super::Real {
	fn from(i: NonNegativeInteger) -> Self {
		Self::Rational(i.into())
	}
}

impl From<NonNegativeInteger> for super::Rational {
	fn from(i: NonNegativeInteger) -> Self {
		i.0.into()
	}
}

impl From<NonNegativeInteger> for super::Decimal {
	fn from(i: NonNegativeInteger) -> Self {
		i.0.into()
	}
}

impl From<NonNegativeInteger> for super::Integer {
	fn from(i: NonNegativeInteger) -> Self {
		i.0
	}
}
