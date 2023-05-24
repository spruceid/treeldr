use unicode_segmentation::UnicodeSegmentation;

use crate::ty;

pub trait MaxGrapheme<T> {
	fn check(&self, value: &T) -> bool;
}

impl MaxGrapheme<ty::NonNegativeInteger> for String {
	fn check(&self, value: &ty::NonNegativeInteger) -> bool {
		match value.as_ref().try_into() {
			Ok(max) => self.as_str().graphemes(true).count() <= max,
			Err(_) => true,
		}
	}
}
