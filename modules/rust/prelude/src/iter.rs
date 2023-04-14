use std::marker::PhantomData;

use crate::{Id, Provider};

pub struct Fetch<'a, C: ?Sized, T, I> {
	context: &'a C,
	iter: I,
	t: PhantomData<T>,
}

impl<'a, C: ?Sized, T, I> Fetch<'a, C, T, I> {
	pub fn new(context: &'a C, iter: I) -> Self {
		Self {
			context,
			iter,
			t: PhantomData,
		}
	}
}

impl<'a, 'd, C: ?Sized, T, I, D: 'd> Iterator for Fetch<'a, C, T, I>
where
	I: Iterator<Item = &'d Id<D>>,
	C: Provider<D, T>,
	T: 'a,
{
	type Item = &'a T;

	fn next(&mut self) -> Option<Self::Item> {
		self.iter.next().map(|id| self.context.get(&id.0).unwrap())
	}
}
