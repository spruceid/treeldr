use crate::ty;

pub trait MinLength<T> {
	fn check(&self, value: &T) -> bool;
}

impl MinLength<ty::NonNegativeInteger> for String {
	fn check(&self, value: &ty::NonNegativeInteger) -> bool {
		match value.as_ref().try_into() {
			Ok(min) => self.len() >= min,
			Err(_) => false,
		}
	}
}
