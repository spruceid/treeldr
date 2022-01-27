use crate::syntax::Location;
use std::collections::BTreeSet;
use std::ops::{Deref, DerefMut};

/// Cause.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Cause {
	/// Explicitly caused by the given source.
	Explicit(Location),

	/// Implicitly caused by the given source.
	Implicit(Location),
}

impl Cause {
	pub fn is_explicit(&self) -> bool {
		matches!(self, Self::Explicit(_))
	}

	pub fn is_implicit(&self) -> bool {
		matches!(self, Self::Implicit(_))
	}

	pub fn source(&self) -> Location {
		match self {
			Self::Explicit(s) => *s,
			Self::Implicit(s) => *s,
		}
	}

	pub fn into_implicit(self) -> Self {
		Self::Implicit(self.source())
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Caused<T> {
	t: T,
	cause: Option<Cause>,
}

impl<T> Caused<T> {
	pub fn new(t: T, cause: Option<Cause>) -> Self {
		Self { t, cause }
	}

	pub fn inner(&self) -> &T {
		&self.t
	}

	pub fn inner_mut(&mut self) -> &mut T {
		&mut self.t
	}

	pub fn cause(&self) -> Option<Cause> {
		self.cause
	}

	pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Caused<U> {
		Caused::new(f(self.t), self.cause)
	}
}

impl<T> Deref for Caused<T> {
	type Target = T;

	fn deref(&self) -> &T {
		self.inner()
	}
}

impl<T> DerefMut for Caused<T> {
	fn deref_mut(&mut self) -> &mut T {
		self.inner_mut()
	}
}

#[derive(Default)]
pub struct Causes {
	set: BTreeSet<Cause>,
}

impl Causes {
	pub fn new() -> Self {
		Self::default()
	}

	/// Adds a new cause.
	pub fn add(&mut self, cause: Cause) {
		self.set.insert(cause);
	}

	/// Picks the preferred cause, unless there are no causes.
	pub fn preferred(&self) -> Option<Cause> {
		self.set.iter().next().cloned()
	}

	pub fn iter(&self) -> impl '_ + Iterator<Item = Cause> {
		self.set.iter().cloned()
	}

	pub fn map(&self, f: impl Fn(Cause) -> Cause) -> Self {
		Self {
			set: self.set.iter().cloned().map(f).collect(),
		}
	}
}

impl From<Cause> for Causes {
	fn from(cause: Cause) -> Self {
		let mut causes = Self::new();
		causes.add(cause);
		causes
	}
}

impl From<Option<Cause>> for Causes {
	fn from(cause: Option<Cause>) -> Self {
		let mut causes = Self::new();
		if let Some(cause) = cause {
			causes.add(cause);
		}
		causes
	}
}

pub struct WithCauses<T> {
	t: T,
	causes: Causes,
}

impl<T> WithCauses<T> {
	pub fn new(t: T, causes: impl Into<Causes>) -> Self {
		Self {
			t,
			causes: causes.into(),
		}
	}

	pub fn causes(&self) -> &Causes {
		&self.causes
	}

	pub fn inner(&self) -> &T {
		&self.t
	}

	pub fn inner_mut(&mut self) -> &mut T {
		&mut self.t
	}
}

impl<T> Deref for WithCauses<T> {
	type Target = T;

	fn deref(&self) -> &T {
		self.inner()
	}
}

impl<T> DerefMut for WithCauses<T> {
	fn deref_mut(&mut self) -> &mut T {
		self.inner_mut()
	}
}
