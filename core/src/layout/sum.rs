use crate::WithCauses;
use shelves::Ref;

/// Sum type.
pub struct Sum<F> {
	name: WithCauses<String, F>,
	options: Vec<WithCauses<Ref<super::Definition<F>>, F>>,
}

impl<F> Sum<F> {
	pub fn new(
		name: WithCauses<String, F>,
		options: Vec<WithCauses<Ref<super::Definition<F>>, F>>,
	) -> Self {
		Self { name, options }
	}

	pub fn name(&self) -> &str {
		&self.name
	}

	pub fn options(&self) -> &[WithCauses<Ref<super::Definition<F>>, F>] {
		&self.options
	}
}
