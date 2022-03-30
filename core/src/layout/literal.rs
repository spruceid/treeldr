use crate::MaybeSet;

pub mod regexp;

pub use regexp::RegExp;

/// Literal value layout.
pub struct Literal<F> {
	/// Layout name.
	///
	/// If no name is set, it must be inlined.
	name: MaybeSet<String, F>,

	/// Regular expression defining the members of the layout.
	regexp: RegExp,
}

impl<F> Literal<F> {
	pub fn new(regexp: RegExp, name: MaybeSet<String, F>) -> Self {
		Self { name, regexp }
	}

	pub fn name(&self) -> Option<&str> {
		self.name.as_deref()
	}

	pub fn regexp(&self) -> &RegExp {
		&self.regexp
	}
}
