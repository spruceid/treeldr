use super::{Integer, Rational};
use crate::vocab::StrippedLiteral;
use std::fmt;

/// Decimal number.
///
/// This is wrapper around rational numbers with a finite decimal
/// representation.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Decimal(Rational);

impl Decimal {
	pub fn as_rational(&self) -> &Rational {
		&self.0
	}

	pub fn into_rational(self) -> Rational {
		self.0
	}

	pub fn is_integer(&self) -> bool {
		self.0.is_integer()
	}

	pub fn as_integer(&self) -> Option<&Integer> {
		self.0.as_integer()
	}

	pub fn literal(&self) -> StrippedLiteral {
		match self.as_integer() {
			Some(i) => i.literal(),
			None => {
				use crate::vocab::{Term, Xsd};
				StrippedLiteral::TypedString(
					self.0.lexical_decimal().unwrap().into_string().into(),
					Term::Xsd(Xsd::Decimal),
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
