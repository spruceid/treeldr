use crate::{prop, Ref, WithCauses};

/// Restricted type.
pub enum Restricted<F> {
	/// Restricted on property.
	Property(RestrictedProperty<F>),
}

/// Property restriction.
pub enum PropertyRestriction<F> {
	/// Range restriction.
	Range(Range<F>),

	/// Cardinality restriction.
	Cardinality(Cardinality),
}

/// Property range restriction.
pub enum Range<F> {
	/// At least one value must be an instance of the given type.
	Any(Ref<super::Definition<F>>),

	/// All the values must be instances of the given type.
	All(Ref<super::Definition<F>>),
}

/// Property cardinality restriction.
pub enum Cardinality {
	/// The property must have at least the given number of values.
	AtLeast(u32),

	/// The property must have at most the given number of values.
	AtMost(u32),

	/// The property must have exactly the given number of values.
	Exactly(u32),
}

/// Type restricted on a property.
///
/// Puts a restriction on a given property.
/// A restricted type is a subset of the domain of the property which
/// includes every instance for which the given property satisfies the
/// given restriction.
pub struct RestrictedProperty<F> {
	/// Property on witch the restriction is placed.
	prop: WithCauses<Ref<prop::Definition<F>>, F>,

	/// Restriction.
	restriction: PropertyRestriction<F>,
}

impl<F> RestrictedProperty<F> {
	pub fn property(&self) -> Ref<prop::Definition<F>> {
		*self.prop
	}

	pub fn restriction(&self) -> &PropertyRestriction<F> {
		&self.restriction
	}
}
