use derivative::Derivative;
use std::marker::PhantomData;

/// Reference to an element of the context.
#[derive(Derivative)]
#[derivative(
	Clone(bound = ""),
	Copy(bound = ""),
	PartialEq(bound = ""),
	Eq(bound = ""),
	Hash(bound = ""),
	PartialOrd(bound = ""),
	Ord(bound = ""),
	Debug(bound = "")
)]
pub struct Ref<T>(usize, PhantomData<T>);

impl<T> Ref<T> {
	pub(crate) fn new(index: usize) -> Self {
		Self(index, PhantomData)
	}

	fn index(&self) -> usize {
		self.0
	}
}

pub struct Collection<T> {
	items: Vec<T>,
}

impl<T> Collection<T> {
	pub fn new() -> Self {
		Self { items: Vec::new() }
	}

	pub fn get(&self, r: Ref<T>) -> Option<&T> {
		self.items.get(r.index())
	}

	pub fn get_mut(&mut self, r: Ref<T>) -> Option<&mut T> {
		self.items.get_mut(r.index())
	}

	pub fn insert(&mut self, v: T) -> Ref<T> {
		let r = Ref::new(self.items.len());
		self.items.push(v);
		r
	}

	pub fn iter(&self) -> impl Iterator<Item = (Ref<T>, &T)> {
		self.items.iter().enumerate().map(|(i, t)| (Ref::new(i), t))
	}
}

impl<T> Default for Collection<T> {
	fn default() -> Self {
		Self::new()
	}
}
