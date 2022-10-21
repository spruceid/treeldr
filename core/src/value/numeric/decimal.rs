use super::{Integer, Rational};
use crate::{vocab::StrippedLiteral, IriIndex};
use num::BigInt;
use std::fmt;

/// Decimal number.
///
/// This is wrapper around rational numbers with a finite decimal
/// representation.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Decimal(Rational);

impl Decimal {
	/// Create a new decimal number from its fractional representation,
	/// without checking that the input rational is indeed a decimal number.
	///
	/// ## Safety
	///
	/// The input rational must have a finite decimal representation.
	pub unsafe fn new_unchecked(r: Rational) -> Self {
		Self(r)
	}

	pub fn is_zero(&self) -> bool {
		self.0.is_zero()
	}

	pub fn is_integer(&self) -> bool {
		self.0.is_integer()
	}

	pub fn as_integer(&self) -> Option<&Integer> {
		self.0.as_integer()
	}

	pub fn is_negative(&self) -> bool {
		self.0.is_negative()
	}

	pub fn as_rational(&self) -> &Rational {
		&self.0
	}

	pub fn into_rational(self) -> Rational {
		self.0
	}

	pub fn into_parts(self) -> (Integer, Integer) {
		self.0.into_parts()
	}

	pub fn into_numer(self) -> Integer {
		self.0.into_numer()
	}

	pub fn into_denum(self) -> Integer {
		self.0.into_denum()
	}

	pub fn literal(&self) -> StrippedLiteral {
		match self.as_integer() {
			Some(i) => i.literal(),
			None => {
				use crate::vocab::{Term, Xsd};
				StrippedLiteral::TypedString(
					self.0.lexical_decimal().unwrap().into_string().into(),
					IriIndex::Iri(Term::Xsd(Xsd::Decimal)),
				)
			}
		}
	}
}

impl fmt::Display for Decimal {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.0.lexical_decimal().unwrap().fmt(f)
	}
}

impl From<BigInt> for Decimal {
	fn from(i: BigInt) -> Self {
		Self(i.into())
	}
}

impl From<Decimal> for super::Real {
	fn from(d: Decimal) -> Self {
		Self::Rational(d.into())
	}
}

impl From<Decimal> for super::Rational {
	fn from(d: Decimal) -> Self {
		d.0
	}
}

impl TryFrom<Decimal> for super::Integer {
	type Error = Decimal;

	fn try_from(d: Decimal) -> Result<Self, Self::Error> {
		if d.is_integer() {
			Ok(d.into_numer())
		} else {
			Err(d)
		}
	}
}

impl TryFrom<Decimal> for super::NonNegativeInteger {
	type Error = Decimal;

	fn try_from(d: Decimal) -> Result<Self, Self::Error> {
		if d.is_integer() && !d.is_negative() {
			Ok(unsafe { Self::new_unchecked(d.into_numer()) })
		} else {
			Err(d)
		}
	}
}
