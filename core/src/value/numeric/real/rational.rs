use crate::{vocab::StrippedLiteral, IriIndex};
use num::{BigInt, BigRational, Signed, Zero};
use std::fmt;
use xsd_types::{Decimal, Integer, NoDecimalRepresentation, NonNegativeInteger};

lazy_static::lazy_static! {
	static ref TEN: BigInt = 10u32.into();
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Rational(BigRational);

impl Rational {
	pub fn numer(&self) -> &Integer {
		Integer::from_bigint_ref(self.0.numer())
	}

	pub fn denom(&self) -> &Integer {
		Integer::from_bigint_ref(self.0.denom())
	}

	pub fn into_parts(self) -> (Integer, Integer) {
		let (n, d) = self.0.into();
		(n.into(), d.into())
	}

	pub fn into_numer(self) -> Integer {
		self.into_parts().0
	}

	pub fn into_denum(self) -> Integer {
		self.into_parts().1
	}

	pub fn is_zero(&self) -> bool {
		self.0.is_zero()
	}

	pub fn is_integer(&self) -> bool {
		self.0.is_integer()
	}

	pub fn is_non_negative_integer(&self) -> bool {
		self.0.is_integer() && !self.0.is_negative()
	}

	pub fn as_integer(&self) -> Option<&Integer> {
		if self.is_integer() {
			Some(self.numer())
		} else {
			None
		}
	}

	pub fn into_integer(self) -> Result<Integer, Self> {
		if self.is_integer() {
			Ok(self.into_numer())
		} else {
			Err(self)
		}
	}

	pub fn into_non_negative_integer(self) -> Result<NonNegativeInteger, Self> {
		if self.is_non_negative_integer() {
			Ok(unsafe { NonNegativeInteger::new_unchecked(self.into_numer().into()) })
		} else {
			Err(self)
		}
	}

	pub fn is_negative(&self) -> bool {
		self.0.is_negative()
	}

	pub fn is_decimal(&self) -> bool {
		xsd_types::is_decimal(&self.0)
	}

	pub fn literal(&self) -> StrippedLiteral {
		use crate::vocab::{Owl, Term, Xsd};
		match xsd_types::decimal_lexical_representation(&self.0) {
			Some(decimal) => StrippedLiteral::TypedString(
				decimal.into_string().into(),
				IriIndex::Iri(Term::Xsd(Xsd::Decimal)),
			),
			None => StrippedLiteral::TypedString(
				self.0.to_string().into(),
				IriIndex::Iri(Term::Owl(Owl::Rational)),
			),
		}
	}
}

impl fmt::Display for Rational {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(f)
	}
}

impl From<BigRational> for Rational {
	fn from(r: BigRational) -> Self {
		Self(r)
	}
}

impl From<BigInt> for Rational {
	fn from(i: BigInt) -> Self {
		Self(i.into())
	}
}

impl From<Decimal> for Rational {
	fn from(value: Decimal) -> Self {
		let n: BigRational = value.into();
		n.into()
	}
}

impl TryFrom<Rational> for super::Decimal {
	type Error = Rational;

	fn try_from(r: Rational) -> Result<Self, Self::Error> {
		match super::Decimal::try_from(r.0) {
			Ok(d) => Ok(d),
			Err(NoDecimalRepresentation(r)) => Err(Rational(r)),
		}
	}
}

impl TryFrom<Rational> for Integer {
	type Error = Rational;

	fn try_from(r: Rational) -> Result<Self, Self::Error> {
		if r.is_integer() {
			Ok(r.into_numer())
		} else {
			Err(r)
		}
	}
}

impl TryFrom<Rational> for NonNegativeInteger {
	type Error = Rational;

	fn try_from(r: Rational) -> Result<Self, Self::Error> {
		if r.is_integer() && !r.is_negative() {
			Ok(r.into_numer().try_into().unwrap())
		} else {
			Err(r)
		}
	}
}
