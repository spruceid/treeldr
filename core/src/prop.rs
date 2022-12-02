use crate::{
	component, layout, list, multiple,
	node::{self, BindingValueRef},
	ty,
	vocab::{self, Term},
	BlankIdIndex, Id, IriIndex, Multiple, Ref, ResourceType, TId,
};
use contextual::DisplayWithContext;
use derivative::Derivative;
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
	pub fn id(&self) -> Id {
		match self {
			Self::Resource(p) => Id::Iri(IriIndex::Iri(p.term())),
			Self::Other(id) => *id,
		}
	}

	pub fn name(&self) -> PropertyName {
		match self {
			Self::Resource(b) => PropertyName::Resource(b.name()),
			Self::Other(id) => PropertyName::Other(*id),
		}
	}

	pub fn expect_type(&self) -> bool {
		match self {
			Self::Resource(p) => p.expect_type(),
			Self::Other(_) => false,
		}
	}

	pub fn expect_layout(&self) -> bool {
		match self {
			Self::Resource(p) => p.expect_layout(),
			Self::Other(_) => false,
		}
	}
}

impl ResourceType for Property {
	const TYPE: crate::Type = crate::Type::Resource(Some(node::Type::Property(None)));

	fn check<M>(resource: &crate::node::Definition<M>) -> bool {
		resource.is_property()
	}
}

impl<'a, M> Ref<'a, Property, M> {
	pub fn as_property(&self) -> &'a Meta<Definition<M>, M> {
		self.as_resource().as_property().unwrap()
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
		Self::Resource(node::Property::Component(component::Property::Layout(
			layout::Property::Description(p),
		)))
	}
}

impl From<component::formatted::Property> for Property {
	fn from(p: component::formatted::Property) -> Self {
		Self::Resource(node::Property::Component(component::Property::Formatted(p)))
	}
}

impl From<layout::field::Property> for Property {
	fn from(p: layout::field::Property) -> Self {
		Self::Resource(node::Property::Component(component::Property::Formatted(
			component::formatted::Property::LayoutField(p),
		)))
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
	Required,
}

impl RdfProperty {
	pub fn term(&self) -> vocab::Term {
		use vocab::{Rdfs, Schema};
		match self {
			Self::Domain => Term::Rdfs(Rdfs::Domain),
			Self::Range => Term::Rdfs(Rdfs::Range),
			Self::Required => Term::Schema(Schema::ValueRequired),
		}
	}

	pub fn name(&self) -> &'static str {
		match self {
			Self::Domain => "domain",
			Self::Range => "range",
			Self::Required => "value requirement",
		}
	}

	pub fn expect_type(&self) -> bool {
		matches!(self, Self::Domain | Self::Range)
	}

	pub fn expect_layout(&self) -> bool {
		false
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Type {
	FunctionalProperty,
}

impl Type {
	/// Checks if this is a subclass of `other`.
	pub fn is_subclass_of(&self, _other: Self) -> bool {
		false
	}

	pub fn term(&self) -> Term {
		match self {
			Self::FunctionalProperty => Term::Owl(vocab::Owl::FunctionalProperty),
		}
	}
}

/// Property definition.
#[derive(Debug)]
pub struct Definition<M> {
	domain: Multiple<TId<crate::Type>, M>,
	range: Multiple<TId<crate::Type>, M>,
	required: Meta<bool, M>,
	functional: Meta<bool, M>,
}

impl<M> Definition<M> {
	pub fn new(
		domain: Multiple<TId<crate::Type>, M>,
		range: Multiple<TId<crate::Type>, M>,
		required: Meta<bool, M>,
		functional: Meta<bool, M>,
	) -> Self {
		Self {
			domain,
			range,
			required,
			functional,
		}
	}

	pub fn range(&self) -> &Multiple<TId<crate::Type>, M> {
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

	pub fn bindings(&self) -> Bindings<M> {
		ClassBindings {
			domain: self.domain.iter(),
			range: self.range.iter(),
			required: if self.is_required() {
				Some(&self.required)
			} else {
				None
			},
		}
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

pub enum ClassBinding {
	Domain(TId<crate::Type>),
	Range(TId<crate::Type>),
	Required(bool),
}

pub type Binding = ClassBinding;

impl ClassBinding {
	pub fn property(&self) -> RdfProperty {
		match self {
			Self::Domain(_) => RdfProperty::Domain,
			Self::Range(_) => RdfProperty::Range,
			Self::Required(_) => RdfProperty::Required,
		}
	}

	pub fn value<'a, M>(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::Domain(v) => BindingValueRef::Type(*v),
			Self::Range(v) => BindingValueRef::Type(*v),
			Self::Required(v) => BindingValueRef::Boolean(*v),
		}
	}
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct ClassBindings<'a, M> {
	domain: multiple::Iter<'a, TId<crate::Type>, M>,
	range: multiple::Iter<'a, TId<crate::Type>, M>,
	required: Option<&'a Meta<bool, M>>,
}

pub type Bindings<'a, M> = ClassBindings<'a, M>;

impl<'a, M> Iterator for ClassBindings<'a, M> {
	type Item = Meta<ClassBinding, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.domain
			.next()
			.map(|m| m.into_cloned_value().map(ClassBinding::Domain))
			.or_else(|| {
				self.range
					.next()
					.map(|m| m.into_cloned_value().map(ClassBinding::Range))
					.or_else(|| {
						self.required
							.take()
							.map(|m| m.borrow().into_cloned_value().map(ClassBinding::Required))
					})
			})
	}
}
