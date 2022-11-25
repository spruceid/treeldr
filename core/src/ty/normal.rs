use super::Properties;
use crate::{metadata::Merge, Property, TId};
use derivative::Derivative;

/// Normal type.
#[derive(Debug, Derivative)]
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
	pub fn insert_property(&mut self, prop_ref: TId<Property>, metadata: M)
	where
		M: Clone + Merge,
	{
		self.properties.insert(prop_ref, None, metadata);
	}
}
