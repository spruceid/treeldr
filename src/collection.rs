use std::marker::PhantomData;
use derivative::Derivative;

/// Reference to an element of the context.
#[derive(Derivative)]
#[derivative(Clone(bound=""), Copy(bound=""), PartialEq(bound=""), Eq(bound=""), Hash(bound=""), PartialOrd(bound=""), Ord(bound=""), Debug(bound=""))]
pub struct Ref<T>(usize, PhantomData<T>);

impl<T> Ref<T> {
	fn index(&self) -> usize {
		self.0
	}
}

pub struct Collection<T> {
	items: Vec<T>
}

impl<T> Collection<T> {
	pub fn new() -> Self {
		Self {
			items: Vec::new()
		}
	}

	pub fn get(&self, r: Ref<T>) -> Option<&T> {
		self.items.get(r.index())
	}
}

impl<T> Default for Collection<T> {
	fn default() -> Self {
		Self::new()
	}
}