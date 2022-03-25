use crate::WithCauses;

pub mod regexp;

pub use regexp::RegExp;

/// Literal value layout.
pub struct Literal<F> {
	/// Layout name.
	name: WithCauses<String, F>,

	/// Regular expression defining the members of the layout.
	regexp: WithCauses<RegExp, F>,
}

impl<F> Literal<F> {
	pub fn name(&self) -> &str {
		&self.name
	}

	pub fn regexp(&self) -> &RegExp {
		&self.regexp
	}
}
