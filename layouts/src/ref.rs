use std::marker::PhantomData;

use educe::Educe;

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
#[repr(transparent)]
pub struct Ref<T, R = rdf_types::Term>(R, PhantomData<T>);

impl<R: Copy, T> Copy for Ref<T, R> {}

impl<T, R> Ref<T, R> {
	pub fn new_ref(id: &R) -> &Self {
		unsafe {
			// SAFETY: `Ref` uses `repr(transparent)` over `R`.
			std::mem::transmute(id)
		}
	}

	pub fn new(id: R) -> Self {
		Self(id, PhantomData)
	}

	pub fn id(&self) -> &R {
		&self.0
	}

	pub fn into_id(self) -> R {
		self.0
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
