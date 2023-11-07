pub mod format;
pub mod graph;
pub mod layout;
pub mod pattern;
pub mod regexp;
pub mod utils;

use std::marker::PhantomData;

use educe::Educe;
pub use format::Format;
pub use graph::{Dataset, Graph};
pub use layout::Layout;
pub use pattern::Pattern;
pub use regexp::RegExp;

/// Typed RDF resource identifier.
#[derive(Educe)]
#[educe(
	Debug(bound = "R: std::fmt::Debug"),
	Clone(bound = "R: Clone"),
	PartialEq(bound = "R: PartialEq"),
	Eq(bound = "R: Eq"),
	PartialOrd(bound = "R: PartialOrd"),
	Ord(bound = "R: Ord"),
	Hash(bound = "R: std::hash::Hash")
)]
pub struct Ref<R, T>(R, PhantomData<T>);

impl<R: Copy, T> Copy for Ref<R, T> {}

impl<R, T> Ref<R, T> {
	pub fn id(&self) -> &R {
		&self.0
	}

	pub fn casted<U>(&self) -> Ref<R, U>
	where
		R: Clone,
	{
		Ref(self.0.clone(), PhantomData)
	}
}

pub trait GetFromContext<C, R>: Sized {
	type Target<'c>
	where
		C: 'c,
		R: 'c;

	fn get_from_context<'c>(context: &'c C, r: &Ref<R, Self>) -> Option<Self::Target<'c>>;
}

pub struct Context<R> {
	r: PhantomData<R>,
}

impl<R> Context<R> {
	pub fn layout(&self, id: &R) -> Option<&Layout<R>> {
		todo!()
	}

	pub fn get<T: GetFromContext<Self, R>>(&self, r: &Ref<R, T>) -> Option<T::Target<'_>> {
		T::get_from_context(self, r)
	}
}
