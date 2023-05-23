use crate::ty;

pub trait MinInclusive<T> {
	fn check(&self, value: &T) -> bool;
}

impl MinInclusive<ty::Integer> for ty::Integer {
	fn check(&self, value: &ty::Integer) -> bool {
		self >= value
	}
}
