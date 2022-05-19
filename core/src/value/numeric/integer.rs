use crate::vocab::StrippedLiteral;
use num::Zero;
use std::fmt;

/// Integer number.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Integer(num::BigInt);

impl Integer {
	pub fn zero() -> Self {
		Self(num::BigInt::zero())
	}

	pub fn is_zero(&self) -> bool {
		self.0.is_zero()
	}

	pub fn literal(&self) -> StrippedLiteral {
		use crate::vocab::{Term, Xsd};
		StrippedLiteral::TypedString(self.0.to_string().into(), Term::Xsd(Xsd::Integer))
	}
}

impl fmt::Display for Integer {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(f)
	}
}
