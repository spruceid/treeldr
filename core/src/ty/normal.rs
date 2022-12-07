use super::Properties;
use crate::{metadata::Merge, Property, TId, Multiple, Type};
use derivative::Derivative;

/// Normal type.
#[derive(Debug, Derivative)]
pub struct Normal<M> {
	/// RDF Syntax `subClassOf` property.
	/// 
	/// Only direct super classes are listed.
	sub_class_of: Multiple<TId<Type>, M>,

	/// Properties.
	properties: Properties<M>,
}

impl<M> Normal<M> {
	/// Create a new normal type.
	/// 
	/// The `sub_class_of` values should contain all and only the direct super classes of this type.
	pub fn new(
		sub_class_of: Multiple<TId<Type>, M>,
	) -> Self {
		Self {
			sub_class_of,
			properties: Properties::default()
		}
	}

	pub fn sub_class_of(&self) -> &Multiple<TId<Type>, M> {
		&self.sub_class_of
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
