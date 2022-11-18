use crate::{TId, ResourceType, Id, node, vocab, BlankIdIndex, IriIndex, ty, layout, list, component, Multiple};
use contextual::DisplayWithContext;
use locspan::Meta;
use rdf_types::Vocabulary;
use std::fmt;

/// Node property.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Property {
	BuiltIn(BuiltIn),
	Other(Id),
}

impl Property {
	pub fn name(&self) -> PropertyName {
		match self {
			Self::BuiltIn(b) => PropertyName::BuiltIn(b.name()),
			Self::Other(id) => PropertyName::Other(*id),
		}
	}
}

impl ResourceType for Property {
	const TYPE: crate::Type = crate::Type::Property(None);

	fn check<M>(resource: &crate::node::Definition<M>) -> bool {
		resource.is_property()
	}
}

impl From<BuiltIn> for Property {
	fn from(b: BuiltIn) -> Self {
		Self::BuiltIn(b)
	}
}

impl From<node::Property> for Property {
	fn from(p: node::Property) -> Self {
		Self::BuiltIn(BuiltIn::Resource(p))
	}
}

impl From<ty::Property> for Property {
	fn from(p: ty::Property) -> Self {
		Self::BuiltIn(BuiltIn::Class(p))
	}
}

impl From<ty::data::Property> for Property {
	fn from(p: ty::data::Property) -> Self {
		Self::BuiltIn(BuiltIn::Class(ty::Property::Datatype(p)))
	}
}

impl From<ty::restriction::Property> for Property {
	fn from(p: ty::restriction::Property) -> Self {
		Self::BuiltIn(BuiltIn::Class(ty::Property::Restriction(p)))
	}
}

impl From<RdfProperty> for Property {
	fn from(p: RdfProperty) -> Self {
		Self::BuiltIn(BuiltIn::Property(p))
	}
}

impl From<component::Property> for Property {
	fn from(p: component::Property) -> Self {
		Self::BuiltIn(BuiltIn::Component(p))
	}
}

impl From<layout::Property> for Property {
	fn from(p: layout::Property) -> Self {
		Self::BuiltIn(BuiltIn::Component(component::Property::Layout(p)))
	}
}

impl From<layout::DescriptionProperty> for Property {
	fn from(p: layout::DescriptionProperty) -> Self {
		Self::BuiltIn(BuiltIn::Component(component::Property::Layout(layout::Property::Description(p))))
	}
}

impl From<component::formatted::Property> for Property {
	fn from(p: component::formatted::Property) -> Self {
		Self::BuiltIn(BuiltIn::Component(component::Property::Formatted(p)))
	}
}

impl From<layout::field::Property> for Property {
	fn from(p: layout::field::Property) -> Self {
		Self::BuiltIn(BuiltIn::Component(component::Property::Formatted(component::formatted::Property::LayoutField(p))))
	}
}

impl From<list::Property> for Property {
	fn from(p: list::Property) -> Self {
		Self::BuiltIn(BuiltIn::List(p))
	}
}

/// Built in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BuiltIn {
	Resource(node::Property),
	Class(ty::Property),
	Property(RdfProperty),
	Component(component::Property),
	List(list::Property)
}

impl BuiltIn {
	fn term(&self) -> vocab::Term {
		match self {
			Self::Resource(p) => p.term(),
			Self::Class(p) => p.term(),
			Self::Property(p) => p.term(),
			Self::Component(p) => p.term(),
			Self::List(p) => p.term()
		}
	}

	fn name(&self) -> &'static str {
		match self {
			Self::Resource(p) => p.name(),
			Self::Class(p) => p.name(),
			Self::Property(p) => p.name(),
			Self::Component(p) => p.name(),
			Self::List(p) => p.name()
		}
	}
}

impl fmt::Display for BuiltIn {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.term().iri().fmt(f)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RdfProperty {
	Domain,
	Range,
	Required
}

impl RdfProperty {
	fn term(&self) -> vocab::Term {
		use vocab::{Term, Rdfs, Schema};
		match self {
			Self::Domain => Term::Rdfs(Rdfs::Domain),
			Self::Range => Term::Rdfs(Rdfs::Range),
			Self::Required => Term::Schema(Schema::ValueRequired)
		}
	}

	fn name(&self) -> &'static str {
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
	BuiltIn(&'static str),
	Other(Id),
}

impl<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>> DisplayWithContext<V> for PropertyName {
	fn fmt_with(&self, context: &V, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::BuiltIn(name) => fmt::Display::fmt(name, f),
			Self::Other(id) => id.fmt_with(context, f),
		}
	}
}