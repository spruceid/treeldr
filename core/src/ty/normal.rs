use super::Properties;
use crate::{metadata, prop};
use derivative::Derivative;
use shelves::Ref;

/// Normal type.
#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct Normal<M> {
	/// Properties.
	properties: Properties<M>,
}

impl<M> Normal<M> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn properties(&self) -> &Properties<M> {
		&self.properties
	}

	/// Insert a property.
	pub fn insert_property(&mut self, prop_ref: Ref<prop::Definition<M>>, metadata: M)
	where
		M: metadata::Merge,
	{
		self.properties.insert(prop_ref, None, metadata);
	}
}
