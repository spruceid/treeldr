use derivative::Derivative;
use std::collections::BTreeSet;
use std::ops::{Deref, DerefMut};
use locspan::Location;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Caused<T, F> {
	t: T,
	cause: Option<Location<F>>,
}

impl<T, F> Caused<T, F> {
	pub fn new(t: T, cause: Option<Location<F>>) -> Self {
		Self { t, cause }
	}

	pub fn inner(&self) -> &T {
		&self.t
	}

	pub fn inner_mut(&mut self) -> &mut T {
		&mut self.t
	}

	pub fn cause(&self) -> Option<&Location<F>> {
		self.cause.as_ref()
	}

	pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Caused<U, F> {
		Caused::new(f(self.t), self.cause)
	}
}

impl<T, F> Deref for Caused<T, F> {
	type Target = T;

	fn deref(&self) -> &T {
		self.inner()
	}
}

impl<T, F> DerefMut for Caused<T, F> {
	fn deref_mut(&mut self) -> &mut T {
		self.inner_mut()
	}
}

#[derive(Derivative)]
#[derivative(Clone, Default(bound=""))]
pub struct Causes<F> {
	set: BTreeSet<Location<F>>,
}

impl<F> Causes<F> {
	pub fn new() -> Self {
		Self::default()
	}

	/// Adds a new cause.
	pub fn add(&mut self, cause: Location<F>) where F: Ord {
		self.set.insert(cause);
	}

	/// Picks the preferred cause, unless there are no causes.
	pub fn preferred(&self) -> Option<&Location<F>> {
		self.set.iter().next()
	}

	pub fn iter(&self) -> impl Iterator<Item = &Location<F>> {
		self.set.iter()
	}

	pub fn map(&self, f: impl Fn(Location<F>) -> Location<F>) -> Self where F: Clone + Ord {
		Self {
			set: self.set.iter().cloned().map(f).collect(),
		}
	}
}

impl<F: Ord> From<Location<F>> for Causes<F> {
	fn from(cause: Location<F>) -> Self {
		let mut causes = Self::new();
		causes.add(cause);
		causes
	}
}

impl<F: Ord> From<Option<Location<F>>> for Causes<F> {
	fn from(cause: Option<Location<F>>) -> Self {
		let mut causes = Self::new();
		if let Some(cause) = cause {
			causes.add(cause);
		}
		causes
	}
}

pub struct WithCauses<T, F> {
	t: T,
	causes: Causes<F>,
}

impl<T, F> WithCauses<T, F> {
	pub fn new(t: T, causes: impl Into<Causes<F>>) -> Self {
		Self {
			t,
			causes: causes.into(),
		}
	}

	pub fn causes(&self) -> &Causes<F> {
		&self.causes
	}

	pub fn inner(&self) -> &T {
		&self.t
	}

	pub fn inner_mut(&mut self) -> &mut T {
		&mut self.t
	}
}

impl<T, F> Deref for WithCauses<T, F> {
	type Target = T;

	fn deref(&self) -> &T {
		self.inner()
	}
}

impl<T, F> DerefMut for WithCauses<T, F> {
	fn deref_mut(&mut self) -> &mut T {
		self.inner_mut()
	}
}
