use super::Properties;
use crate::{prop, Ref, WithCauses};

/// Type restricted on a property.
///
/// Puts a restriction on a given property.
/// A restricted type is a subset of the domain of the property which
/// includes every instance for which the given property satisfies the
/// given restriction.
pub struct Restriction<F> {
	properties: Properties<F>,
}

impl<F> Restriction<F> {
	pub fn new(
		prop: WithCauses<Ref<prop::Definition<F>>, F>,
		restriction: prop::Restrictions<F>,
	) -> Self
	where
		F: Ord,
	{
		let mut properties = Properties::none();
		let (prop, causes) = prop.into_parts();
		properties.insert(prop, Some(restriction), causes);

		Self { properties }
	}

	pub fn properties(&self) -> &Properties<F> {
		&self.properties
	}

	pub fn property(&self) -> Ref<prop::Definition<F>> {
		self.properties.included().next().unwrap().property()
	}

	pub fn restrictions(&self) -> &prop::Restrictions<F> {
		self.properties.included().next().unwrap().restrictions()
	}
}
