use super::Properties;
use crate::{prop, TId, metadata::Merge, vocab};
use locspan::Meta;

/// Type restricted on a property.
///
/// Puts a restriction on a given property.
/// A restricted type is a subset of the domain of the property which
/// includes every instance for which the given property satisfies the
/// given restriction.
#[derive(Debug)]
pub struct Restriction<M> {
	properties: Properties<M>,
}

impl<M> Restriction<M> {
	pub fn new(
		Meta(prop, causes): Meta<TId<crate::Property>, M>,
		restriction: Meta<prop::Restriction, M>
	) -> Self
	where
		M: Clone + Merge,
	{
		let mut properties = Properties::none();

		properties.insert(prop, Some(prop::Restrictions::singleton(restriction)), causes);

		Self { properties }
	}

	pub fn properties(&self) -> &Properties<M> {
		&self.properties
	}

	pub fn property(&self) -> TId<crate::Property> {
		self.properties.included().next().unwrap().property()
	}

	pub fn restrictions(&self) -> &prop::Restrictions<M> {
		self.properties.included().next().unwrap().restrictions()
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Property {
	OnProperty,
	AllValuesFrom,
	SomeValuesFrom
}

impl Property {
	pub fn term(&self) -> vocab::Term {
		use vocab::{Term, Owl};
		match self {
			Self::OnProperty => Term::Owl(Owl::OnProperty),
			Self::AllValuesFrom => Term::Owl(Owl::AllValuesFrom),
			Self::SomeValuesFrom => Term::Owl(Owl::SomeValuesFrom),
		}
	}

	pub fn name(&self) -> &'static str {
		match self {
			Self::OnProperty => "restricted property",
			Self::AllValuesFrom => "all values from range",
			Self::SomeValuesFrom => "some values from range"
		}
	}
}