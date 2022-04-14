use super::Properties;
use crate::{prop, Causes};
use derivative::Derivative;
use shelves::Ref;
use std::collections::HashMap;

/// Normal type.
#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct Normal<F> {
	/// Properties.
	properties: HashMap<Ref<prop::Definition<F>>, Causes<F>>,
}

impl<F> Normal<F> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn properties(&self) -> Properties<F> {
		Properties(self.properties.iter())
	}

	/// Insert a property.
	pub fn insert_property(
		&mut self,
		prop_ref: Ref<prop::Definition<F>>,
		causes: impl Into<Causes<F>>,
	) where
		F: Ord,
	{
		self.properties.insert(prop_ref, causes.into());
	}
}
