use super::Properties;
use crate::{prop, Causes};
use derivative::Derivative;
use shelves::Ref;

/// Normal type.
#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct Normal<F> {
	/// Properties.
	properties: Properties<F>,
}

impl<F> Normal<F> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn properties(&self) -> &Properties<F> {
		&self.properties
	}

	/// Insert a property.
	pub fn insert_property(
		&mut self,
		prop_ref: Ref<prop::Definition<F>>,
		causes: impl Into<Causes<F>>,
	) where
		F: Ord,
	{
		self.properties.insert(prop_ref, None, causes.into());
	}
}
