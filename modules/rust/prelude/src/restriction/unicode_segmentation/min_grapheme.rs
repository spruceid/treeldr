use unicode_segmentation::UnicodeSegmentation;

use crate::ty;

pub trait MinGrapheme<T> {
	fn check(&self, value: &T) -> bool;
}

impl MinGrapheme<ty::NonNegativeInteger> for String {
	fn check(&self, value: &ty::NonNegativeInteger) -> bool {
		match value.as_ref().try_into() {
			Ok(min) => self.as_str().graphemes(true).count() >= min,
			Err(_) => false,
		}
	}
}
