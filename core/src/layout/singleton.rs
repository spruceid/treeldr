use crate::{Causes, Name, WithCauses, Value};
use locspan::Location;

/// Enum layout.
#[derive(Clone)]
pub struct Singleton<F> {
	name: WithCauses<Name, F>,
	value: WithCauses<Value, F>
}

impl<F> Singleton<F> {
	pub fn new(name: WithCauses<Name, F>, value: WithCauses<Value, F>) -> Self {
		Self { name, value }
	}

	pub fn name(&self) -> &Name {
		&self.name
	}

	pub fn into_name(self) -> WithCauses<Name, F> {
		self.name
	}

	pub fn name_causes(&self) -> &Causes<F> {
		self.name.causes()
	}

	pub fn set_name(&mut self, new_name: Name, cause: Option<Location<F>>) -> WithCauses<Name, F>
	where
		F: Ord,
	{
		std::mem::replace(&mut self.name, WithCauses::new(new_name, cause))
	}

	pub fn value(&self) -> &Value {
		self.value.inner()
	}
}