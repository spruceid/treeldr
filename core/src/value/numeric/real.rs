use crate::vocab::StrippedLiteral;

/// Real number value.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Real {
	Rational(super::Rational),
}

impl Real {
	pub fn literal(&self) -> StrippedLiteral {
		match self {
			Self::Rational(r) => r.literal(),
		}
	}
}
