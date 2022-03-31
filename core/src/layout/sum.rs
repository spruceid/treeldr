use crate::{WithCauses, vocab::Name};
use shelves::Ref;

/// Sum type.
pub struct Sum<F> {
	name: WithCauses<Name, F>,
	options: Vec<Ref<super::Definition<F>>>,
}

impl<F> Sum<F> {
	pub fn new(
		name: WithCauses<Name, F>,
		options: Vec<Ref<super::Definition<F>>>,
	) -> Self {
		Self { name, options }
	}

	pub fn name(&self) -> &Name {
		&self.name
	}

	pub fn options(&self) -> &[Ref<super::Definition<F>>] {
		&self.options
	}
}
