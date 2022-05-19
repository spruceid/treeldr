use crate::vocab::StrippedLiteral;
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Double(ordered_float::NotNan<f32>);

impl Double {
	pub const NEG_INFINITY: Self =
		unsafe { Self(ordered_float::NotNan::new_unchecked(f32::NEG_INFINITY)) };
	pub const INFINITY: Self = unsafe { Self(ordered_float::NotNan::new_unchecked(f32::INFINITY)) };

	pub fn new(f: ordered_float::NotNan<f32>) -> Self {
		Self(f)
	}

	pub fn literal(&self) -> StrippedLiteral {
		use crate::vocab::{Term, Xsd};
		StrippedLiteral::TypedString(self.0.to_string().into(), Term::Xsd(Xsd::Double))
	}
}

impl fmt::Display for Double {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(f)
	}
}
