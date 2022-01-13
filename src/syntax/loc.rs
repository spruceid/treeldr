use std::fmt;
use super::Span;
use crate::Source;

/// Located syntax node.
#[derive(Clone, Copy, Debug)]
pub struct Loc<T> {
	/// Data.
	t: T,

	/// Source position.
	source: Source
}

impl<T> Loc<T> {
	pub fn new(t: T, source: Source) -> Self {
		Self {
			t, source
		}
	}

	pub fn inner(&self) -> &T {
		&self.t
	}

	pub fn source(&self) -> Source {
		self.source
	}

	pub fn span(&self) -> Span {
		self.source.span()
	}

	pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Loc<U> {
		Loc {
			t: f(self.t),
			source: self.source
		}
	}

	pub fn into_parts(self) -> (T, Source) {
		(self.t, self.source)
	}

	pub fn into_inner(self) -> T {
		self.t
	}
}

impl<T: fmt::Display> fmt::Display for Loc<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.t.fmt(f)
	}
}