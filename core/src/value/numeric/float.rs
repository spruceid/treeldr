use crate::{vocab::StrippedLiteral, IriIndex};
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Float(ordered_float::NotNan<f32>);

impl Float {
	pub const NEG_INFINITY: Self =
		unsafe { Self(ordered_float::NotNan::new_unchecked(f32::NEG_INFINITY)) };
	pub const INFINITY: Self = unsafe { Self(ordered_float::NotNan::new_unchecked(f32::INFINITY)) };

	pub fn new(f: ordered_float::NotNan<f32>) -> Self {
		Self(f)
	}

	pub fn literal(&self) -> StrippedLiteral {
		use crate::vocab::{Term, Xsd};
		StrippedLiteral::TypedString(
			self.0.to_string().into(),
			IriIndex::Iri(Term::Xsd(Xsd::Float)),
		)
	}

	pub fn unwrap(self) -> ordered_float::NotNan<f32> {
		self.0
	}
}

impl fmt::Display for Float {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(f)
	}
}

impl TryFrom<f32> for Float {
	type Error = ordered_float::FloatIsNan;

	fn try_from(f: f32) -> Result<Self, Self::Error> {
		Ok(Self(f.try_into()?))
	}
}
