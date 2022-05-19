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

impl TryFrom<Real> for super::Rational {
	type Error = Real;

	fn try_from(r: Real) -> Result<Self, Self::Error> {
		match r {
			Real::Rational(r) => Ok(r),
		}
	}
}

impl TryFrom<Real> for super::Decimal {
	type Error = Real;

	fn try_from(r: Real) -> Result<Self, Self::Error> {
		match r {
			Real::Rational(r) => r.try_into().map_err(Real::Rational),
		}
	}
}

impl TryFrom<Real> for super::Integer {
	type Error = Real;

	fn try_from(r: Real) -> Result<Self, Self::Error> {
		match r {
			Real::Rational(r) => r.try_into().map_err(Real::Rational),
		}
	}
}

impl TryFrom<Real> for super::NonNegativeInteger {
	type Error = Real;

	fn try_from(r: Real) -> Result<Self, Self::Error> {
		match r {
			Real::Rational(r) => r.try_into().map_err(Real::Rational),
		}
	}
}
