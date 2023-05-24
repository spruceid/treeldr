use crate::ty;

pub trait MaxLength<T> {
	fn check(&self, value: &T) -> bool;
}

impl MaxLength<ty::NonNegativeInteger> for String {
	fn check(&self, value: &ty::NonNegativeInteger) -> bool {
		match value.as_ref().try_into() {
			Ok(max) => self.len() <= max,
			Err(_) => true,
		}
	}
}
