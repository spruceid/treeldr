use std::{convert::Infallible, marker::PhantomData};

use crate::Ref;

pub struct Ids<I>(I);

impl<I> Ids<I> {
	pub fn new(iter: I) -> Self {
		Self(iter)
	}
}

impl<'r, Id: 'r, I: Iterator<Item = &'r crate::Id<Id>>> Iterator for Ids<I> {
	type Item = Ref<'r, Id, Infallible>;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next().map(|id| Ref::Id(&id.0))
	}
}

pub struct Values<Id, I>(I, PhantomData<Id>);

impl<Id, I> Values<Id, I> {
	pub fn new(iter: I) -> Self {
		Self(iter, PhantomData)
	}
}

impl<'r, Id: 'r, T: 'r, I: Iterator<Item = &'r T>> Iterator for Values<Id, I> {
	type Item = Ref<'r, Id, T>;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next().map(Ref::Value)
	}
}
