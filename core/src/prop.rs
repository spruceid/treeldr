use crate::{
	component, layout, list,
	node::{self, BindingValueRef},
	property_values, ty,
	vocab::{self, Owl, Rdf, Rdfs, Schema, Term, TreeLdr, Xsd},
	BlankIdIndex, FunctionalPropertyValue, Id, IriIndex, PropertyValues, Ref, ResourceType, TId,
};
use contextual::DisplayWithContext;
use locspan::Meta;
use rdf_types::Vocabulary;
use std::fmt;

/// Non built-in property.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct UnknownProperty;

pub type SubPropertyId = UnknownProperty;

/// Node property.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Property {
	Resource(node::Property),
	Other(TId<UnknownProperty>),
}

impl Property {
	pub fn id(&self) -> Id {
		match self {
			Self::Resource(p) => p.id(),
			Self::Other(id) => id.id(),
		}
	}

	pub fn name(&self) -> PropertyName {
		match self {
			Self::Resource(b) => b.name(),
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

impl From<Term> for Property {
	fn from(t: Term) -> Self {
		match t {
			Term::TreeLdr(TreeLdr::Self_) => Self::Resource(node::Property::Self_(None)),
			Term::Rdf(Rdf::Type) => Self::Resource(node::Property::Type(None)),
			Term::Rdfs(Rdfs::Label) => Self::Resource(node::Property::Label(None)),
			Term::Rdfs(Rdfs::Comment) => Self::Resource(node::Property::Comment(None)),
			Term::Rdfs(Rdfs::SubClassOf) => {
				Self::Resource(node::Property::Class(ty::Property::SubClassOf(None)))
			}
			Term::Owl(Owl::UnionOf) => {
				Self::Resource(node::Property::Class(ty::Property::UnionOf(None)))
			}
			Term::Owl(Owl::IntersectionOf) => {
				Self::Resource(node::Property::Class(ty::Property::IntersectionOf(None)))
			}
			Term::Owl(Owl::OnDatatype) => Self::Resource(node::Property::Class(
				ty::Property::Datatype(ty::data::Property::OnDatatype(None)),
			)),
			Term::Owl(Owl::WithRestrictions) => Self::Resource(node::Property::Class(
				ty::Property::Datatype(ty::data::Property::WithRestrictions(None)),
			)),
			Term::Owl(Owl::OnProperty) => Self::Resource(node::Property::Class(
				ty::Property::Restriction(ty::restriction::Property::OnProperty(None)),
			)),
			Term::Owl(Owl::AllValuesFrom) => Self::Resource(node::Property::Class(
				ty::Property::Restriction(ty::restriction::Property::AllValuesFrom(None)),
			)),
			Term::Owl(Owl::SomeValuesFrom) => Self::Resource(node::Property::Class(
				ty::Property::Restriction(ty::restriction::Property::SomeValuesFrom(None)),
			)),
			Term::Owl(Owl::MinCardinality) => Self::Resource(node::Property::Class(
				ty::Property::Restriction(ty::restriction::Property::MinCardinality(None)),
			)),
			Term::Owl(Owl::MaxCardinality) => Self::Resource(node::Property::Class(
				ty::Property::Restriction(ty::restriction::Property::MaxCardinality(None)),
			)),
			Term::Owl(Owl::Cardinality) => Self::Resource(node::Property::Class(
				ty::Property::Restriction(ty::restriction::Property::Cardinality(None)),
			)),
			Term::Xsd(Xsd::MinInclusive) => Self::Resource(node::Property::DatatypeRestriction(
				ty::data::restriction::Property::MinInclusive(None),
			)),
			Term::Xsd(Xsd::MinExclusive) => Self::Resource(node::Property::DatatypeRestriction(
				ty::data::restriction::Property::MinExclusive(None),
			)),
			Term::Xsd(Xsd::MaxInclusive) => Self::Resource(node::Property::DatatypeRestriction(
				ty::data::restriction::Property::MaxInclusive(None),
			)),
			Term::Xsd(Xsd::MaxExclusive) => Self::Resource(node::Property::DatatypeRestriction(
				ty::data::restriction::Property::MaxExclusive(None),
			)),
			Term::Xsd(Xsd::MinLength) => Self::Resource(node::Property::DatatypeRestriction(
				ty::data::restriction::Property::MinLength(None),
			)),
			Term::Xsd(Xsd::MaxLength) => Self::Resource(node::Property::DatatypeRestriction(
				ty::data::restriction::Property::MaxLength(None),
			)),
			Term::Xsd(Xsd::Pattern) => Self::Resource(node::Property::DatatypeRestriction(
				ty::data::restriction::Property::Pattern(None),
			)),
			Term::Rdfs(Rdfs::Domain) => {
				Self::Resource(node::Property::Property(RdfProperty::Domain(None)))
			}
			Term::Rdfs(Rdfs::Range) => {
				Self::Resource(node::Property::Property(RdfProperty::Range(None)))
			}
			Term::Schema(Schema::ValueRequired) => {
				Self::Resource(node::Property::Property(RdfProperty::Required(None)))
			}
			Term::TreeLdr(TreeLdr::Name) => {
				Self::Resource(node::Property::Component(component::Property::Name(None)))
			}
			Term::TreeLdr(TreeLdr::LayoutFor) => Self::Resource(node::Property::Component(
				component::Property::Layout(layout::Property::For(None)),
			)),
			Term::TreeLdr(TreeLdr::Alias) => {
				Self::Resource(node::Property::Component(component::Property::Layout(
					layout::Property::Description(layout::DescriptionProperty::Alias(None)),
				)))
			}
			Term::TreeLdr(TreeLdr::Array) => {
				Self::Resource(node::Property::Component(component::Property::Layout(
					layout::Property::Description(layout::DescriptionProperty::Array(None)),
				)))
			}
			Term::TreeLdr(TreeLdr::DerivedFrom) => {
				Self::Resource(node::Property::Component(component::Property::Layout(
					layout::Property::Description(layout::DescriptionProperty::DerivedFrom(None)),
				)))
			}
			Term::TreeLdr(TreeLdr::Fields) => {
				Self::Resource(node::Property::Component(component::Property::Layout(
					layout::Property::Description(layout::DescriptionProperty::Fields(None)),
				)))
			}
			Term::TreeLdr(TreeLdr::OneOrMany) => {
				Self::Resource(node::Property::Component(component::Property::Layout(
					layout::Property::Description(layout::DescriptionProperty::OneOrMany(None)),
				)))
			}
			Term::TreeLdr(TreeLdr::Option) => {
				Self::Resource(node::Property::Component(component::Property::Layout(
					layout::Property::Description(layout::DescriptionProperty::Option(None)),
				)))
			}
			Term::TreeLdr(TreeLdr::Reference) => {
				Self::Resource(node::Property::Component(component::Property::Layout(
					layout::Property::Description(layout::DescriptionProperty::Reference(None)),
				)))
			}
			Term::TreeLdr(TreeLdr::Required) => {
				Self::Resource(node::Property::Component(component::Property::Layout(
					layout::Property::Description(layout::DescriptionProperty::Required(None)),
				)))
			}
			Term::TreeLdr(TreeLdr::Set) => {
				Self::Resource(node::Property::Component(component::Property::Layout(
					layout::Property::Description(layout::DescriptionProperty::Set(None)),
				)))
			}
			Term::TreeLdr(TreeLdr::Enumeration) => {
				Self::Resource(node::Property::Component(component::Property::Layout(
					layout::Property::Description(layout::DescriptionProperty::Variants(None)),
				)))
			}
			Term::TreeLdr(TreeLdr::IntersectionOf) => Self::Resource(node::Property::Component(
				component::Property::Layout(layout::Property::IntersectionOf(None)),
			)),
			Term::TreeLdr(TreeLdr::WithRestrictions) => Self::Resource(node::Property::Component(
				component::Property::Layout(layout::Property::WithRestrictions(None)),
			)),
			Term::TreeLdr(TreeLdr::ArrayListFirst) => Self::Resource(node::Property::Component(
				component::Property::Layout(layout::Property::ArrayListFirst(None)),
			)),
			Term::TreeLdr(TreeLdr::ArrayListRest) => Self::Resource(node::Property::Component(
				component::Property::Layout(layout::Property::ArrayListRest(None)),
			)),
			Term::TreeLdr(TreeLdr::ArrayListNil) => Self::Resource(node::Property::Component(
				component::Property::Layout(layout::Property::ArrayListNil(None)),
			)),
			Term::TreeLdr(TreeLdr::Format) => Self::Resource(node::Property::Component(
				component::Property::Formatted(component::formatted::Property::Format(None)),
			)),
			Term::TreeLdr(TreeLdr::FieldFor) => {
				Self::Resource(node::Property::Component(component::Property::Formatted(
					component::formatted::Property::LayoutField(layout::field::Property::For(None)),
				)))
			}
			Term::TreeLdr(TreeLdr::ExclusiveMaximum) => {
				Self::Resource(node::Property::LayoutRestriction(
					layout::restriction::Property::ExclusiveMaximum(None),
				))
			}
			Term::TreeLdr(TreeLdr::ExclusiveMinimum) => {
				Self::Resource(node::Property::LayoutRestriction(
					layout::restriction::Property::ExclusiveMinimum(None),
				))
			}
			Term::TreeLdr(TreeLdr::InclusiveMaximum) => {
				Self::Resource(node::Property::LayoutRestriction(
					layout::restriction::Property::InclusiveMaximum(None),
				))
			}
			Term::TreeLdr(TreeLdr::InclusiveMinimum) => {
				Self::Resource(node::Property::LayoutRestriction(
					layout::restriction::Property::InclusiveMinimum(None),
				))
			}
			Term::TreeLdr(TreeLdr::MaxCardinality) => {
				Self::Resource(node::Property::LayoutRestriction(
					layout::restriction::Property::MaxCardinality(None),
				))
			}
			Term::TreeLdr(TreeLdr::MaxLength) => Self::Resource(node::Property::LayoutRestriction(
				layout::restriction::Property::MaxLength(None),
			)),
			Term::TreeLdr(TreeLdr::MinCardinality) => {
				Self::Resource(node::Property::LayoutRestriction(
					layout::restriction::Property::MinCardinality(None),
				))
			}
			Term::TreeLdr(TreeLdr::MinLength) => Self::Resource(node::Property::LayoutRestriction(
				layout::restriction::Property::MinLength(None),
			)),
			Term::TreeLdr(TreeLdr::Pattern) => Self::Resource(node::Property::LayoutRestriction(
				layout::restriction::Property::Pattern(None),
			)),
			Term::Rdf(Rdf::First) => {
				Self::Resource(node::Property::List(list::Property::First(None)))
			}
			Term::Rdf(Rdf::Rest) => {
				Self::Resource(node::Property::List(list::Property::Rest(None)))
			}
			t => Self::Other(TId::new(Id::Iri(IriIndex::Iri(t)))),
		}
	}
}

impl From<Id> for Property {
	fn from(id: Id) -> Self {
		match id {
			Id::Iri(IriIndex::Iri(t)) => t.into(),
			id => Self::Other(TId::new(id)),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RdfProperty {
	Domain(Option<TId<UnknownProperty>>),
	Range(Option<TId<UnknownProperty>>),
	SubPropertyOf(Option<TId<UnknownProperty>>),
	Required(Option<TId<UnknownProperty>>),
}

impl RdfProperty {
	pub fn id(&self) -> Id {
		match self {
			Self::Domain(None) => Id::Iri(IriIndex::Iri(Term::Rdfs(Rdfs::Domain))),
			Self::Domain(Some(p)) => p.id(),
			Self::Range(None) => Id::Iri(IriIndex::Iri(Term::Rdfs(Rdfs::Range))),
			Self::Range(Some(p)) => p.id(),
			Self::SubPropertyOf(None) => Id::Iri(IriIndex::Iri(Term::Rdfs(Rdfs::SubPropertyOf))),
			Self::SubPropertyOf(Some(p)) => p.id(),
			Self::Required(None) => Id::Iri(IriIndex::Iri(Term::Schema(Schema::ValueRequired))),
			Self::Required(Some(p)) => p.id(),
		}
	}

	pub fn term(&self) -> Option<vocab::Term> {
		match self {
			Self::Domain(None) => Some(Term::Rdfs(Rdfs::Domain)),
			Self::Range(None) => Some(Term::Rdfs(Rdfs::Range)),
			Self::SubPropertyOf(None) => Some(Term::Rdfs(Rdfs::SubPropertyOf)),
			Self::Required(None) => Some(Term::Schema(Schema::ValueRequired)),
			_ => None,
		}
	}

	pub fn name(&self) -> PropertyName {
		match self {
			Self::Domain(None) => PropertyName::Resource("domain"),
			Self::Domain(Some(p)) => PropertyName::Other(*p),
			Self::Range(None) => PropertyName::Resource("range"),
			Self::Range(Some(p)) => PropertyName::Other(*p),
			Self::SubPropertyOf(None) => PropertyName::Resource("super property"),
			Self::SubPropertyOf(Some(p)) => PropertyName::Other(*p),
			Self::Required(None) => PropertyName::Resource("value requirement"),
			Self::Required(Some(p)) => PropertyName::Other(*p),
		}
	}

	pub fn expect_type(&self) -> bool {
		matches!(self, Self::Domain(_) | Self::Range(_))
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
	domain: PropertyValues<TId<crate::Type>, M>,
	range: PropertyValues<TId<crate::Type>, M>,
	required: FunctionalPropertyValue<bool, M>,
	functional: Meta<bool, M>,
}

impl<M> Definition<M> {
	pub fn new(
		domain: PropertyValues<TId<crate::Type>, M>,
		range: PropertyValues<TId<crate::Type>, M>,
		required: FunctionalPropertyValue<bool, M>,
		functional: Meta<bool, M>,
	) -> Self {
		Self {
			domain,
			range,
			required,
			functional,
		}
	}

	pub fn range(&self) -> &PropertyValues<TId<crate::Type>, M> {
		&self.range
	}

	pub fn domain(&self) -> &PropertyValues<TId<crate::Type>, M> {
		&self.domain
	}

	pub fn is_required(&self) -> bool {
		self.required.is_some_and(|v| *v)
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
			required: self.required.iter(),
		}
	}
}

pub enum PropertyName {
	Resource(&'static str),
	Other(TId<UnknownProperty>),
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
	Domain(Option<TId<UnknownProperty>>, TId<crate::Type>),
	Range(Option<TId<UnknownProperty>>, TId<crate::Type>),
	Required(Option<TId<UnknownProperty>>, bool),
}

pub type Binding = ClassBinding;

impl ClassBinding {
	pub fn domain(&self) -> Option<Type> {
		None
	}

	pub fn property(&self) -> RdfProperty {
		match self {
			Self::Domain(p, _) => RdfProperty::Domain(*p),
			Self::Range(p, _) => RdfProperty::Range(*p),
			Self::Required(p, _) => RdfProperty::Required(*p),
		}
	}

	pub fn value<'a, M>(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::Domain(_, v) => BindingValueRef::Type(*v),
			Self::Range(_, v) => BindingValueRef::Type(*v),
			Self::Required(_, v) => BindingValueRef::SchemaBoolean(*v),
		}
	}
}

pub struct ClassBindings<'a, M> {
	domain: property_values::non_functional::Iter<'a, TId<crate::Type>, M>,
	range: property_values::non_functional::Iter<'a, TId<crate::Type>, M>,
	required: property_values::functional::Iter<'a, bool, M>,
}

pub type Bindings<'a, M> = ClassBindings<'a, M>;

impl<'a, M> Iterator for ClassBindings<'a, M> {
	type Item = Meta<ClassBinding, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.domain
			.next()
			.map(|m| m.into_cloned_class_binding(ClassBinding::Domain))
			.or_else(|| {
				self.range
					.next()
					.map(|m| m.into_cloned_class_binding(ClassBinding::Range))
					.or_else(|| {
						self.required
							.next()
							.map(|m| m.into_cloned_class_binding(ClassBinding::Required))
					})
			})
	}
}
