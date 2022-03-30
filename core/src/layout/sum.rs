use crate::WithCauses;
use shelves::Ref;

/// Sum type.
pub struct Sum<F> {
	name: WithCauses<String, F>,
	options: Vec<Ref<super::Definition<F>>>,
}

impl<F> Sum<F> {
	pub fn new(
		name: WithCauses<String, F>,
		options: Vec<Ref<super::Definition<F>>>,
	) -> Self {
		Self { name, options }
	}

	pub fn name(&self) -> &str {
		&self.name
	}

	pub fn options(&self) -> &[Ref<super::Definition<F>>] {
		&self.options
	}
}
