pub mod format;
pub mod graph;
pub mod layout;
pub mod matching;
pub mod pattern;
pub mod regexp;
pub mod utils;
pub mod value;

use std::marker::PhantomData;

use educe::Educe;
pub use format::Format;
pub use graph::{Dataset, Graph};
pub use layout::Layout;
pub use matching::Matching;
pub use pattern::Pattern;
pub use regexp::RegExp;
pub use value::{Literal, TypedLiteral, TypedValue, Value};

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
pub struct Ref<T, R>(R, PhantomData<T>);

impl<R: Copy, T> Copy for Ref<T, R> {}

impl<T, R> Ref<T, R> {
	pub fn id(&self) -> &R {
		&self.0
	}

	pub fn cast<U>(self) -> Ref<U, R> {
		Ref(self.0, PhantomData)
	}

	pub fn casted<U>(&self) -> Ref<U, R>
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

	fn get_from_context<'c>(context: &'c C, r: &Ref<Self, R>) -> Option<Self::Target<'c>>;
}

pub struct Context<R> {
	r: PhantomData<R>,
}

impl<R> Context<R> {
	pub fn layout(&self, id: &R) -> Option<&Layout<R>> {
		todo!()
	}

	pub fn get<T: GetFromContext<Self, R>>(&self, r: &Ref<T, R>) -> Option<T::Target<'_>> {
		T::get_from_context(self, r)
	}
}
