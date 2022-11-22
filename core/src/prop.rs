use crate::{TId, ResourceType, Id, node, vocab::{self, Term}, BlankIdIndex, IriIndex, ty, layout, list, component, Multiple};
use contextual::DisplayWithContext;
use locspan::Meta;
use rdf_types::Vocabulary;
use std::fmt;

/// Node property.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Property {
	Resource(node::Property),
	Other(Id),
}

impl Property {
	pub fn name(&self) -> PropertyName {
		match self {
			Self::Resource(b) => PropertyName::Resource(b.name()),
			Self::Other(id) => PropertyName::Other(*id),
		}
	}
}

impl ResourceType for Property {
	const TYPE: crate::Type = crate::Type::Resource(Some(node::Type::Property(None)));

	fn check<M>(resource: &crate::node::Definition<M>) -> bool {
		resource.is_property()
	}
}

impl From<node::Property> for Property {
	fn from(p: node::Property) -> Self {
		Self::Resource(p)
	}
}

impl From<ty::Property> for Property {
	fn from(p: ty::Property) -> Self {
		Self::Resource(node::Property::Class(p))
	}
}

impl From<ty::data::Property> for Property {
	fn from(p: ty::data::Property) -> Self {
		Self::Resource(node::Property::Class(ty::Property::Datatype(p)))
	}
}

impl From<ty::restriction::Property> for Property {
	fn from(p: ty::restriction::Property) -> Self {
		Self::Resource(node::Property::Class(ty::Property::Restriction(p)))
	}
}

impl From<RdfProperty> for Property {
	fn from(p: RdfProperty) -> Self {
		Self::Resource(node::Property::Property(p))
	}
}

impl From<component::Property> for Property {
	fn from(p: component::Property) -> Self {
		Self::Resource(node::Property::Component(p))
	}
}

impl From<layout::Property> for Property {
	fn from(p: layout::Property) -> Self {
		Self::Resource(node::Property::Component(component::Property::Layout(p)))
	}
}

impl From<layout::DescriptionProperty> for Property {
	fn from(p: layout::DescriptionProperty) -> Self {
		Self::Resource(node::Property::Component(component::Property::Layout(layout::Property::Description(p))))
	}
}

impl From<component::formatted::Property> for Property {
	fn from(p: component::formatted::Property) -> Self {
		Self::Resource(node::Property::Component(component::Property::Formatted(p)))
	}
}

impl From<layout::field::Property> for Property {
	fn from(p: layout::field::Property) -> Self {
		Self::Resource(node::Property::Component(component::Property::Formatted(component::formatted::Property::LayoutField(p))))
	}
}

impl From<list::Property> for Property {
	fn from(p: list::Property) -> Self {
		Self::Resource(node::Property::List(p))
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RdfProperty {
	Domain,
	Range,
	Required
}

impl RdfProperty {
	pub fn term(&self) -> vocab::Term {
		use vocab::{Rdfs, Schema};
		match self {
			Self::Domain => Term::Rdfs(Rdfs::Domain),
			Self::Range => Term::Rdfs(Rdfs::Range),
			Self::Required => Term::Schema(Schema::ValueRequired)
		}
	}

	pub fn name(&self) -> &'static str {
		match self {
			Self::Domain => "domain",
			Self::Range => "range",
			Self::Required => "value requirement"
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Type {
	FunctionalProperty
}

impl Type {
	/// Checks if this is a subclass of `other`.
	pub fn is_subclass_of(&self, _other: Self) -> bool {
		false
	}

	pub fn term(&self) -> Term {
		match self {
			Self::FunctionalProperty => Term::Owl(vocab::Owl::FunctionalProperty)
		}
	}
}

/// Property definition.
#[derive(Debug)]
pub struct Definition<M> {
	domain: Multiple<TId<crate::Type>, M>,
	range: Meta<TId<crate::Type>, M>,
	required: Meta<bool, M>,
	functional: Meta<bool, M>
}

impl<M> Definition<M> {
	pub fn new(
		domain: Multiple<TId<crate::Type>, M>,
		range: Meta<TId<crate::Type>, M>,
		required: Meta<bool, M>,
		functional: Meta<bool, M>
	) -> Self {
		Self {
			domain,
			range,
			required,
			functional,
		}
	}

	pub fn range(&self) -> &Meta<TId<crate::Type>, M> {
		&self.range
	}

	pub fn domain(&self) -> &Multiple<TId<crate::Type>, M> {
		&self.domain
	}

	pub fn is_required(&self) -> bool {
		*self.required
	}

	/// Checks if this property is functional,
	/// meaning that it is associated to at most one value.
	pub fn is_functional(&self) -> bool {
		*self.functional
	}
}

pub enum PropertyName {
	Resource(&'static str),
	Other(Id),
}

impl<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>> DisplayWithContext<V> for PropertyName {
	fn fmt_with(&self, context: &V, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Resource(name) => fmt::Display::fmt(name, f),
			Self::Other(id) => id.fmt_with(context, f),
		}
	}
}