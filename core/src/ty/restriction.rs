use super::Properties;
use crate::{metadata, prop, Ref};
use locspan::Meta;

/// Type restricted on a property.
///
/// Puts a restriction on a given property.
/// A restricted type is a subset of the domain of the property which
/// includes every instance for which the given property satisfies the
/// given restriction.
pub struct Restriction<M> {
	properties: Properties<M>,
}

impl<M> Restriction<M> {
	pub fn new(
		Meta(prop, causes): Meta<Ref<prop::Definition<M>>, M>,
		restriction: prop::Restrictions<M>,
	) -> Self
	where
		M: metadata::Merge,
	{
		let mut properties = Properties::none();
		properties.insert(prop, Some(restriction), causes);

		Self { properties }
	}

	pub fn properties(&self) -> &Properties<M> {
		&self.properties
	}

	pub fn property(&self) -> Ref<prop::Definition<M>> {
		self.properties.included().next().unwrap().property()
	}

	pub fn restrictions(&self) -> &prop::Restrictions<M> {
		self.properties.included().next().unwrap().restrictions()
	}
}
