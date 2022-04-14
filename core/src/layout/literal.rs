use crate::{error, vocab::Name, Caused, Error, Id, MaybeSet, WithCauses};
use locspan::Location;

pub mod regexp;

pub use regexp::RegExp;

/// Literal value layout.
#[derive(Clone)]
pub struct Literal<F> {
	/// Layout name.
	name: WithCauses<Name, F>,

	/// Regular expression defining the members of the layout.
	regexp: RegExp,

	/// Should the literal type be inlined in the code?
	should_inline: bool,
}

impl<F> Literal<F> {
	pub fn new(regexp: RegExp, name: WithCauses<Name, F>, should_inline: bool) -> Self {
		Self {
			name,
			regexp,
			should_inline,
		}
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

	pub fn intersected_with(
		self,
		id: Id,
		other: &Self,
		name: MaybeSet<Name, F>,
		cause: Option<&Location<F>>,
	) -> Result<Self, Error<F>>
	where
		F: Clone + Ord,
	{
		if self.regexp == other.regexp {
			Ok(Self {
				name: name.unwrap().unwrap_or(self.name),
				regexp: self.regexp,
				should_inline: self.should_inline && other.should_inline,
			})
		} else {
			Err(Caused::new(
				error::LayoutIntersectionFailed { id }.into(),
				cause.cloned(),
			))
		}
	}
}
