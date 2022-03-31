use crate::{WithCauses, vocab::Name};

pub mod regexp;

pub use regexp::RegExp;

/// Literal value layout.
pub struct Literal<F> {
	/// Layout name.
	name: WithCauses<Name, F>,

	/// Regular expression defining the members of the layout.
	regexp: RegExp,

	/// Should the literal type be inlined in the code?
	should_inline: bool
}

impl<F> Literal<F> {
	pub fn new(regexp: RegExp, name: WithCauses<Name, F>, should_inline: bool) -> Self {
		Self { name, regexp, should_inline }
	}

	pub fn name(&self) -> &Name {
		&self.name
	}

	pub fn regexp(&self) -> &RegExp {
		&self.regexp
	}

	pub fn should_inline(&self) -> bool {
		self.should_inline
	}
}
