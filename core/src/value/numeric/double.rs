use crate::vocab::StrippedLiteral;
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Double(ordered_float::NotNan<f64>);

impl Double {
	pub const NEG_INFINITY: Self =
		unsafe { Self(ordered_float::NotNan::new_unchecked(f64::NEG_INFINITY)) };
	pub const INFINITY: Self = unsafe { Self(ordered_float::NotNan::new_unchecked(f64::INFINITY)) };

	pub fn new(f: ordered_float::NotNan<f64>) -> Self {
		Self(f)
	}

	pub fn literal(&self) -> StrippedLiteral {
		use crate::vocab::{Term, Xsd};
		StrippedLiteral::TypedString(self.0.to_string().into(), Term::Xsd(Xsd::Double))
	}

	pub fn unwrap(self) -> ordered_float::NotNan<f64> {
		self.0
	}
}

impl fmt::Display for Double {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(f)
	}
}

impl TryFrom<f64> for Double {
	type Error = ordered_float::FloatIsNan;

	fn try_from(f: f64) -> Result<Self, Self::Error> {
		Ok(Self(f.try_into()?))
	}
}
