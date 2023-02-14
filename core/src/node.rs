use crate::{
	component, doc, error, layout, list, multiple,
	prop::{self, PropertyName, UnknownProperty},
	property_values, ty,
	vocab::{self, Term},
	Documentation, Error, Id, IriIndex, MetaOption, Multiple, MutableModel, Name, PropertyValues,
	ResourceType, TId,
};
use locspan::Meta;
use xsd_types::NonNegativeInteger;

/// Resource data.
#[derive(Debug, Clone)]
pub struct Data<M> {
	pub id: Id,
	pub metadata: M,
	pub type_: PropertyValues<TId<crate::Type>, M>,
	pub label: PropertyValues<String, M>,
	pub comment: Documentation<M>,
}

impl<M> Data<M> {
	pub fn new(id: Id, metadata: M) -> Self {
		Self {
			id,
			metadata,
			type_: PropertyValues::default(),
			label: PropertyValues::default(),
			comment: Documentation::default(),
		}
	}

	pub fn bindings(&self) -> ClassBindings<M> {
		ClassBindings {
			type_: self.type_.iter(),
			label: self.label.iter(),
			comment: self.comment.iter(),
		}
	}
}

/// Resource.
pub struct Resource;

impl ResourceType for Resource {
	const TYPE: crate::Type = crate::Type::Resource(None);

	fn check<M>(_resource: &self::Definition<M>) -> bool {
		true
	}
}

/// Resource definition.
#[derive(Debug)]
pub struct Definition<M> {
	data: Data<M>,
	ty: MetaOption<ty::Definition<M>, M>,
	property: MetaOption<prop::Definition<M>, M>,
	component: MetaOption<component::Definition<M>, M>,
}

impl<M> Definition<M> {
	pub fn new(
		data: Data<M>,
		ty: MetaOption<ty::Definition<M>, M>,
		property: MetaOption<prop::Definition<M>, M>,
		component: MetaOption<component::Definition<M>, M>,
	) -> Self {
		Self {
			data,
			ty,
			property,
			component,
		}
	}

	pub fn id(&self) -> Id {
		self.data.id
	}

	pub fn type_(&self) -> &PropertyValues<TId<crate::Type>, M> {
		&self.data.type_
	}

	pub fn label(&self) -> &PropertyValues<String, M> {
		&self.data.label
	}

	pub fn preferred_label(&self) -> Option<&str> {
		self.data.label.first().map(|m| m.value.as_str())
	}

	pub fn comment(&self) -> &Documentation<M> {
		&self.data.comment
	}

	pub fn is_type(&self) -> bool {
		self.ty.is_some()
	}

	pub fn is_datatype(&self, model: &MutableModel<M>) -> bool {
		self.ty
			.value()
			.map(|v| v.is_datatype(model))
			.unwrap_or(false)
	}

	pub fn is_property(&self) -> bool {
		self.property.is_some()
	}

	pub fn is_layout(&self) -> bool {
		self.component
			.value()
			.map(component::Definition::is_layout)
			.unwrap_or(false)
	}

	pub fn is_layout_field(&self) -> bool {
		self.component
			.value()
			.map(component::Definition::is_layout_field)
			.unwrap_or(false)
	}

	pub fn is_layout_variant(&self) -> bool {
		self.component
			.value()
			.map(component::Definition::is_layout_variant)
			.unwrap_or(false)
	}

	pub fn as_type(&self) -> Option<&Meta<ty::Definition<M>, M>> {
		self.ty.as_ref()
	}

	pub fn as_property(&self) -> Option<&Meta<prop::Definition<M>, M>> {
		self.property.as_ref()
	}

	pub fn as_component(&self) -> Option<&Meta<component::Definition<M>, M>> {
		self.component.as_ref()
	}

	pub fn as_layout(&self) -> Option<&Meta<layout::Definition<M>, M>> {
		self.component
			.value()
			.and_then(component::Definition::as_layout)
	}

	pub fn as_formatted(&self) -> Option<&Meta<component::formatted::Definition<M>, M>> {
		self.component
			.value()
			.and_then(component::Definition::as_formatted)
	}

	pub fn as_layout_field(&self) -> Option<&Meta<layout::field::Definition<M>, M>> {
		self.component
			.value()
			.and_then(component::Definition::as_layout_field)
	}

	pub fn as_layout_variant(&self) -> Option<&Meta<layout::variant::Definition, M>> {
		self.component
			.value()
			.and_then(component::Definition::as_layout_variant)
	}

	pub fn require_layout(&self) -> Result<&Meta<layout::Definition<M>, M>, Error<M>>
	where
		M: Clone,
	{
		self.as_layout().ok_or_else(|| {
			error::NodeInvalidType {
				id: self.data.id,
				expected: crate::Type::Resource(Some(Type::Component(Some(
					component::Type::Layout,
				))))
				.id(),
				found: self.type_().clone(),
			}
			.into()
		})
	}

	pub fn bindings(&self) -> Bindings<M> {
		Bindings {
			data: self.data.bindings(),
			class: self.ty.as_ref().map(|t| t.bindings()).unwrap_or_default(),
			property: self.property.as_ref().map(|p| p.bindings()),
			component: self
				.component
				.as_ref()
				.map(|c| c.bindings())
				.unwrap_or_default(),
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Type {
	Literal,
	Class(Option<ty::SubClass>),
	DatatypeRestriction,
	Property(Option<prop::Type>),
	Component(Option<component::Type>),
	LayoutRestriction,
	List,
}

impl Type {
	/// Checks if this is a subclass of `other`.
	pub fn is_subclass_of(&self, other: Self) -> bool {
		match (self, other) {
			(Self::Class(Some(_)), Self::Class(None)) => true,
			(Self::Class(Some(a)), Self::Class(Some(b))) => a.is_subclass_of(b),
			(Self::Property(Some(_)), Self::Property(None)) => true,
			(Self::Property(Some(a)), Self::Property(Some(b))) => a.is_subclass_of(b),
			(Self::Component(Some(_)), Self::Component(None)) => true,
			(Self::Component(Some(a)), Self::Component(Some(b))) => a.is_subclass_of(b),
			_ => false,
		}
	}

	pub fn term(&self) -> Term {
		match self {
			Self::Literal => Term::Rdfs(vocab::Rdfs::Literal),
			Self::Class(None) => Term::Rdfs(vocab::Rdfs::Class),
			Self::Class(Some(ty)) => ty.term(),
			Self::DatatypeRestriction => Term::Rdfs(vocab::Rdfs::Resource),
			Self::Property(None) => Term::Rdf(vocab::Rdf::Property),
			Self::Property(Some(ty)) => ty.term(),
			Self::Component(None) => Term::TreeLdr(vocab::TreeLdr::Component),
			Self::Component(Some(ty)) => ty.term(),
			Self::LayoutRestriction => Term::TreeLdr(vocab::TreeLdr::LayoutRestriction),
			Self::List => Term::Rdf(vocab::Rdf::List),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Property {
	Self_(Option<TId<UnknownProperty>>),
	Type(Option<TId<UnknownProperty>>),
	Label(Option<TId<UnknownProperty>>),
	Comment(Option<TId<UnknownProperty>>),
	Class(ty::Property),
	DatatypeRestriction(ty::data::restriction::Property),
	Property(prop::RdfProperty),
	Component(component::Property),
	LayoutRestriction(layout::restriction::Property),
	List(list::Property),
}

impl Property {
	pub const ALL: [Self; 56] = [
		Self::Self_(None),
		Self::Type(None),
		Self::Label(None),
		Self::Comment(None),
		Self::Class(ty::Property::UnionOf(None)),
		Self::Class(ty::Property::IntersectionOf(None)),
		Self::Class(ty::Property::SubClassOf(None)),
		Self::Class(ty::Property::Datatype(ty::data::Property::OnDatatype(None))),
		Self::Class(ty::Property::Datatype(
			ty::data::Property::WithRestrictions(None),
		)),
		Self::Class(ty::Property::Restriction(
			ty::restriction::Property::AllValuesFrom(None),
		)),
		Self::Class(ty::Property::Restriction(
			ty::restriction::Property::Cardinality(None),
		)),
		Self::Class(ty::Property::Restriction(
			ty::restriction::Property::MaxCardinality(None),
		)),
		Self::Class(ty::Property::Restriction(
			ty::restriction::Property::MinCardinality(None),
		)),
		Self::Class(ty::Property::Restriction(
			ty::restriction::Property::OnProperty(None),
		)),
		Self::Class(ty::Property::Restriction(
			ty::restriction::Property::SomeValuesFrom(None),
		)),
		Self::Property(prop::RdfProperty::Domain(None)),
		Self::Property(prop::RdfProperty::Range(None)),
		Self::Property(prop::RdfProperty::Required(None)),
		Self::Property(prop::RdfProperty::SubPropertyOf(None)),
		Self::DatatypeRestriction(ty::data::restriction::Property::MaxExclusive(None)),
		Self::DatatypeRestriction(ty::data::restriction::Property::MaxInclusive(None)),
		Self::DatatypeRestriction(ty::data::restriction::Property::MaxLength(None)),
		Self::DatatypeRestriction(ty::data::restriction::Property::MinExclusive(None)),
		Self::DatatypeRestriction(ty::data::restriction::Property::MinInclusive(None)),
		Self::DatatypeRestriction(ty::data::restriction::Property::MinLength(None)),
		Self::DatatypeRestriction(ty::data::restriction::Property::Pattern(None)),
		Self::List(list::Property::First(None)),
		Self::List(list::Property::Rest(None)),
		Self::Component(component::Property::Name(None)),
		Self::Component(component::Property::Formatted(
			component::formatted::Property::Format(None),
		)),
		Self::Component(component::Property::Formatted(
			component::formatted::Property::LayoutField(layout::field::Property::For(None)),
		)),
		Self::Component(component::Property::Layout(
			layout::Property::ArrayListFirst(None),
		)),
		Self::Component(component::Property::Layout(layout::Property::ArrayListNil(
			None,
		))),
		Self::Component(component::Property::Layout(
			layout::Property::ArrayListRest(None),
		)),
		Self::Component(component::Property::Layout(layout::Property::For(None))),
		Self::Component(component::Property::Layout(
			layout::Property::IntersectionOf(None),
		)),
		Self::Component(component::Property::Layout(
			layout::Property::WithRestrictions(None),
		)),
		Self::Component(component::Property::Layout(layout::Property::Description(
			layout::DescriptionProperty::Alias(None),
		))),
		Self::Component(component::Property::Layout(layout::Property::Description(
			layout::DescriptionProperty::Array(None),
		))),
		Self::Component(component::Property::Layout(layout::Property::Description(
			layout::DescriptionProperty::DerivedFrom(None),
		))),
		Self::Component(component::Property::Layout(layout::Property::Description(
			layout::DescriptionProperty::Fields(None),
		))),
		Self::Component(component::Property::Layout(layout::Property::Description(
			layout::DescriptionProperty::OneOrMany(None),
		))),
		Self::Component(component::Property::Layout(layout::Property::Description(
			layout::DescriptionProperty::Option(None),
		))),
		Self::Component(component::Property::Layout(layout::Property::Description(
			layout::DescriptionProperty::Reference(None),
		))),
		Self::Component(component::Property::Layout(layout::Property::Description(
			layout::DescriptionProperty::Required(None),
		))),
		Self::Component(component::Property::Layout(layout::Property::Description(
			layout::DescriptionProperty::Set(None),
		))),
		Self::Component(component::Property::Layout(layout::Property::Description(
			layout::DescriptionProperty::Variants(None),
		))),
		Self::LayoutRestriction(layout::restriction::Property::ExclusiveMaximum(None)),
		Self::LayoutRestriction(layout::restriction::Property::ExclusiveMinimum(None)),
		Self::LayoutRestriction(layout::restriction::Property::InclusiveMaximum(None)),
		Self::LayoutRestriction(layout::restriction::Property::InclusiveMinimum(None)),
		Self::LayoutRestriction(layout::restriction::Property::MaxCardinality(None)),
		Self::LayoutRestriction(layout::restriction::Property::MaxLength(None)),
		Self::LayoutRestriction(layout::restriction::Property::MinCardinality(None)),
		Self::LayoutRestriction(layout::restriction::Property::MinLength(None)),
		Self::LayoutRestriction(layout::restriction::Property::Pattern(None)),
	];

	pub fn into_sub_property(self, sub_prop: Option<TId<UnknownProperty>>) -> Self {
		match self {
			Self::Self_(_) => Self::Self_(sub_prop),
			Self::Type(_) => Self::Type(sub_prop),
			Self::Label(_) => Self::Label(sub_prop),
			Self::Comment(_) => Self::Comment(sub_prop),
			Self::Class(ty::Property::UnionOf(_)) => Self::Class(ty::Property::UnionOf(sub_prop)),
			Self::Class(ty::Property::IntersectionOf(_)) => {
				Self::Class(ty::Property::IntersectionOf(sub_prop))
			}
			Self::Class(ty::Property::SubClassOf(_)) => {
				Self::Class(ty::Property::SubClassOf(sub_prop))
			}
			Self::Class(ty::Property::Datatype(ty::data::Property::OnDatatype(_))) => Self::Class(
				ty::Property::Datatype(ty::data::Property::OnDatatype(sub_prop)),
			),
			Self::Class(ty::Property::Datatype(ty::data::Property::WithRestrictions(_))) => {
				Self::Class(ty::Property::Datatype(
					ty::data::Property::WithRestrictions(sub_prop),
				))
			}
			Self::Class(ty::Property::Restriction(ty::restriction::Property::AllValuesFrom(_))) => {
				Self::Class(ty::Property::Restriction(
					ty::restriction::Property::AllValuesFrom(sub_prop),
				))
			}
			Self::Class(ty::Property::Restriction(ty::restriction::Property::Cardinality(_))) => {
				Self::Class(ty::Property::Restriction(
					ty::restriction::Property::Cardinality(sub_prop),
				))
			}
			Self::Class(ty::Property::Restriction(ty::restriction::Property::MaxCardinality(
				_,
			))) => Self::Class(ty::Property::Restriction(
				ty::restriction::Property::MaxCardinality(sub_prop),
			)),
			Self::Class(ty::Property::Restriction(ty::restriction::Property::MinCardinality(
				_,
			))) => Self::Class(ty::Property::Restriction(
				ty::restriction::Property::MinCardinality(sub_prop),
			)),
			Self::Class(ty::Property::Restriction(ty::restriction::Property::OnProperty(_))) => {
				Self::Class(ty::Property::Restriction(
					ty::restriction::Property::OnProperty(sub_prop),
				))
			}
			Self::Class(ty::Property::Restriction(ty::restriction::Property::SomeValuesFrom(
				_,
			))) => Self::Class(ty::Property::Restriction(
				ty::restriction::Property::SomeValuesFrom(sub_prop),
			)),
			Self::Property(prop::RdfProperty::Domain(_)) => {
				Self::Property(prop::RdfProperty::Domain(sub_prop))
			}
			Self::Property(prop::RdfProperty::Range(_)) => {
				Self::Property(prop::RdfProperty::Range(sub_prop))
			}
			Self::Property(prop::RdfProperty::Required(_)) => {
				Self::Property(prop::RdfProperty::Required(sub_prop))
			}
			Self::Property(prop::RdfProperty::SubPropertyOf(_)) => {
				Self::Property(prop::RdfProperty::SubPropertyOf(sub_prop))
			}
			Self::DatatypeRestriction(ty::data::restriction::Property::MaxExclusive(_)) => {
				Self::DatatypeRestriction(ty::data::restriction::Property::MaxExclusive(sub_prop))
			}
			Self::DatatypeRestriction(ty::data::restriction::Property::MaxInclusive(_)) => {
				Self::DatatypeRestriction(ty::data::restriction::Property::MaxInclusive(sub_prop))
			}
			Self::DatatypeRestriction(ty::data::restriction::Property::MaxLength(_)) => {
				Self::DatatypeRestriction(ty::data::restriction::Property::MaxLength(sub_prop))
			}
			Self::DatatypeRestriction(ty::data::restriction::Property::MinExclusive(_)) => {
				Self::DatatypeRestriction(ty::data::restriction::Property::MinExclusive(sub_prop))
			}
			Self::DatatypeRestriction(ty::data::restriction::Property::MinInclusive(_)) => {
				Self::DatatypeRestriction(ty::data::restriction::Property::MinInclusive(sub_prop))
			}
			Self::DatatypeRestriction(ty::data::restriction::Property::MinLength(_)) => {
				Self::DatatypeRestriction(ty::data::restriction::Property::MinLength(sub_prop))
			}
			Self::DatatypeRestriction(ty::data::restriction::Property::Pattern(_)) => {
				Self::DatatypeRestriction(ty::data::restriction::Property::Pattern(sub_prop))
			}
			Self::List(list::Property::First(_)) => Self::List(list::Property::First(sub_prop)),
			Self::List(list::Property::Rest(_)) => Self::List(list::Property::Rest(sub_prop)),
			Self::Component(component::Property::Name(_)) => {
				Self::Component(component::Property::Name(sub_prop))
			}
			Self::Component(component::Property::Formatted(
				component::formatted::Property::Format(_),
			)) => Self::Component(component::Property::Formatted(
				component::formatted::Property::Format(sub_prop),
			)),
			Self::Component(component::Property::Formatted(
				component::formatted::Property::LayoutField(layout::field::Property::For(_)),
			)) => Self::Component(component::Property::Formatted(
				component::formatted::Property::LayoutField(layout::field::Property::For(sub_prop)),
			)),
			Self::Component(component::Property::Layout(layout::Property::ArrayListFirst(_))) => {
				Self::Component(component::Property::Layout(
					layout::Property::ArrayListFirst(sub_prop),
				))
			}
			Self::Component(component::Property::Layout(layout::Property::ArrayListNil(_))) => {
				Self::Component(component::Property::Layout(layout::Property::ArrayListNil(
					sub_prop,
				)))
			}
			Self::Component(component::Property::Layout(layout::Property::ArrayListRest(_))) => {
				Self::Component(component::Property::Layout(
					layout::Property::ArrayListRest(sub_prop),
				))
			}
			Self::Component(component::Property::Layout(layout::Property::For(_))) => {
				Self::Component(component::Property::Layout(layout::Property::For(sub_prop)))
			}
			Self::Component(component::Property::Layout(layout::Property::IntersectionOf(_))) => {
				Self::Component(component::Property::Layout(
					layout::Property::IntersectionOf(sub_prop),
				))
			}
			Self::Component(component::Property::Layout(layout::Property::WithRestrictions(_))) => {
				Self::Component(component::Property::Layout(
					layout::Property::WithRestrictions(sub_prop),
				))
			}
			Self::Component(component::Property::Layout(layout::Property::Description(
				layout::DescriptionProperty::Alias(_),
			))) => Self::Component(component::Property::Layout(layout::Property::Description(
				layout::DescriptionProperty::Alias(sub_prop),
			))),
			Self::Component(component::Property::Layout(layout::Property::Description(
				layout::DescriptionProperty::Array(_),
			))) => Self::Component(component::Property::Layout(layout::Property::Description(
				layout::DescriptionProperty::Array(sub_prop),
			))),
			Self::Component(component::Property::Layout(layout::Property::Description(
				layout::DescriptionProperty::DerivedFrom(_),
			))) => Self::Component(component::Property::Layout(layout::Property::Description(
				layout::DescriptionProperty::DerivedFrom(sub_prop),
			))),
			Self::Component(component::Property::Layout(layout::Property::Description(
				layout::DescriptionProperty::Fields(_),
			))) => Self::Component(component::Property::Layout(layout::Property::Description(
				layout::DescriptionProperty::Fields(sub_prop),
			))),
			Self::Component(component::Property::Layout(layout::Property::Description(
				layout::DescriptionProperty::OneOrMany(_),
			))) => Self::Component(component::Property::Layout(layout::Property::Description(
				layout::DescriptionProperty::OneOrMany(sub_prop),
			))),
			Self::Component(component::Property::Layout(layout::Property::Description(
				layout::DescriptionProperty::Option(_),
			))) => Self::Component(component::Property::Layout(layout::Property::Description(
				layout::DescriptionProperty::Option(sub_prop),
			))),
			Self::Component(component::Property::Layout(layout::Property::Description(
				layout::DescriptionProperty::Reference(_),
			))) => Self::Component(component::Property::Layout(layout::Property::Description(
				layout::DescriptionProperty::Reference(sub_prop),
			))),
			Self::Component(component::Property::Layout(layout::Property::Description(
				layout::DescriptionProperty::Required(_),
			))) => Self::Component(component::Property::Layout(layout::Property::Description(
				layout::DescriptionProperty::Required(sub_prop),
			))),
			Self::Component(component::Property::Layout(layout::Property::Description(
				layout::DescriptionProperty::Set(_),
			))) => Self::Component(component::Property::Layout(layout::Property::Description(
				layout::DescriptionProperty::Set(sub_prop),
			))),
			Self::Component(component::Property::Layout(layout::Property::Description(
				layout::DescriptionProperty::Variants(_),
			))) => Self::Component(component::Property::Layout(layout::Property::Description(
				layout::DescriptionProperty::Variants(sub_prop),
			))),
			Self::LayoutRestriction(layout::restriction::Property::ExclusiveMaximum(_)) => {
				Self::LayoutRestriction(layout::restriction::Property::ExclusiveMaximum(sub_prop))
			}
			Self::LayoutRestriction(layout::restriction::Property::ExclusiveMinimum(_)) => {
				Self::LayoutRestriction(layout::restriction::Property::ExclusiveMinimum(sub_prop))
			}
			Self::LayoutRestriction(layout::restriction::Property::InclusiveMaximum(_)) => {
				Self::LayoutRestriction(layout::restriction::Property::InclusiveMaximum(sub_prop))
			}
			Self::LayoutRestriction(layout::restriction::Property::InclusiveMinimum(_)) => {
				Self::LayoutRestriction(layout::restriction::Property::InclusiveMinimum(sub_prop))
			}
			Self::LayoutRestriction(layout::restriction::Property::MaxCardinality(_)) => {
				Self::LayoutRestriction(layout::restriction::Property::MaxCardinality(sub_prop))
			}
			Self::LayoutRestriction(layout::restriction::Property::MaxLength(_)) => {
				Self::LayoutRestriction(layout::restriction::Property::MaxLength(sub_prop))
			}
			Self::LayoutRestriction(layout::restriction::Property::MinCardinality(_)) => {
				Self::LayoutRestriction(layout::restriction::Property::MinCardinality(sub_prop))
			}
			Self::LayoutRestriction(layout::restriction::Property::MinLength(_)) => {
				Self::LayoutRestriction(layout::restriction::Property::MinLength(sub_prop))
			}
			Self::LayoutRestriction(layout::restriction::Property::Pattern(_)) => {
				Self::LayoutRestriction(layout::restriction::Property::Pattern(sub_prop))
			}
		}
	}

	pub fn id(&self) -> Id {
		use vocab::{Rdf, Rdfs};
		match self {
			Self::Self_(None) => Id::Iri(IriIndex::Iri(Term::TreeLdr(vocab::TreeLdr::Self_))),
			Self::Self_(Some(p)) => p.id(),
			Self::Type(None) => Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::Type))),
			Self::Type(Some(p)) => p.id(),
			Self::Label(None) => Id::Iri(IriIndex::Iri(Term::Rdfs(Rdfs::Label))),
			Self::Label(Some(p)) => p.id(),
			Self::Comment(None) => Id::Iri(IriIndex::Iri(Term::Rdfs(Rdfs::Comment))),
			Self::Comment(Some(p)) => p.id(),
			Self::Class(p) => p.id(),
			Self::DatatypeRestriction(p) => p.id(),
			Self::Property(p) => p.id(),
			Self::Component(p) => p.id(),
			Self::LayoutRestriction(p) => p.id(),
			Self::List(p) => p.id(),
		}
	}

	pub fn term(&self) -> Option<vocab::Term> {
		use vocab::{Rdf, Rdfs};
		match self {
			Self::Self_(None) => Some(Term::TreeLdr(vocab::TreeLdr::Self_)),
			Self::Type(None) => Some(Term::Rdf(Rdf::Type)),
			Self::Label(None) => Some(Term::Rdfs(Rdfs::Label)),
			Self::Comment(None) => Some(Term::Rdfs(Rdfs::Comment)),
			Self::Class(p) => p.term(),
			Self::DatatypeRestriction(p) => p.term(),
			Self::Property(p) => p.term(),
			Self::Component(p) => p.term(),
			Self::LayoutRestriction(p) => p.term(),
			Self::List(p) => p.term(),
			_ => None,
		}
	}

	pub fn name(&self) -> PropertyName {
		match self {
			Self::Self_(None) => PropertyName::Resource("self reference"),
			Self::Self_(Some(p)) => PropertyName::Other(*p),
			Self::Type(None) => PropertyName::Resource("type"),
			Self::Type(Some(p)) => PropertyName::Other(*p),
			Self::Label(None) => PropertyName::Resource("label"),
			Self::Label(Some(p)) => PropertyName::Other(*p),
			Self::Comment(None) => PropertyName::Resource("comment"),
			Self::Comment(Some(p)) => PropertyName::Other(*p),
			Self::Class(p) => p.name(),
			Self::DatatypeRestriction(p) => p.name(),
			Self::Property(p) => p.name(),
			Self::Component(p) => p.name(),
			Self::LayoutRestriction(p) => p.name(),
			Self::List(p) => p.name(),
		}
	}

	pub fn expect_type(&self) -> bool {
		match self {
			Self::Type(_) => true,
			Self::Class(p) => p.expect_type(),
			Self::DatatypeRestriction(p) => p.expect_type(),
			Self::Property(p) => p.expect_type(),
			Self::Component(p) => p.expect_type(),
			Self::LayoutRestriction(p) => p.expect_type(),
			Self::List(p) => p.expect_type(),
			_ => false,
		}
	}

	pub fn expect_layout(&self) -> bool {
		match self {
			Self::Class(p) => p.expect_layout(),
			Self::DatatypeRestriction(p) => p.expect_layout(),
			Self::Property(p) => p.expect_layout(),
			Self::Component(p) => p.expect_layout(),
			Self::LayoutRestriction(p) => p.expect_layout(),
			Self::List(p) => p.expect_layout(),
			_ => false,
		}
	}
}

pub enum ClassBindingRef<'a> {
	Type(Option<TId<UnknownProperty>>, TId<crate::Type>),
	Label(Option<TId<UnknownProperty>>, &'a str),
	Comment(Option<TId<UnknownProperty>>, &'a str),
}

impl<'a> ClassBindingRef<'a> {
	pub fn into_binding_ref<M>(self) -> BindingRef<'a, M> {
		match self {
			Self::Type(p, t) => BindingRef::Type(p, t),
			Self::Label(p, l) => BindingRef::Label(p, l),
			Self::Comment(p, c) => BindingRef::Comment(p, c),
		}
	}
}

pub struct ClassBindings<'a, M> {
	type_: property_values::non_functional::Iter<'a, TId<crate::Type>, M>,
	label: property_values::non_functional::Iter<'a, String, M>,
	comment: doc::Iter<'a, M>,
}

impl<'a, M> Iterator for ClassBindings<'a, M> {
	type Item = Meta<ClassBindingRef<'a>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.type_
			.next()
			.map(|v| v.into_cloned_class_binding(ClassBindingRef::Type))
			.or_else(|| {
				self.label
					.next()
					.map(|v| v.into_deref_class_binding(ClassBindingRef::Label))
					.or_else(|| {
						self.comment
							.next()
							.map(|v| v.into_deref_class_binding(ClassBindingRef::Comment))
					})
			})
	}
}

pub enum MultipleIdValueRef<'a, T, M> {
	Single(TId<T>),
	PropertyValue(&'a PropertyValues<TId<T>, M>),
	Multiple(&'a Multiple<TId<T>, M>),
}

impl<'a, T, M> MultipleIdValueRef<'a, T, M> {
	pub fn iter(&self) -> MultipleIdValueIter<'a, T, M> {
		match self {
			Self::Single(v) => MultipleIdValueIter::Single(Some(*v)),
			Self::PropertyValue(v) => MultipleIdValueIter::PropertyValue(v.iter()),
			Self::Multiple(v) => MultipleIdValueIter::Multiple(v.iter()),
		}
	}
}

pub enum MultipleIdValueIter<'a, T, M> {
	Single(Option<TId<T>>),
	PropertyValue(property_values::non_functional::Iter<'a, TId<T>, M>),
	Multiple(multiple::Iter<'a, TId<T>, M>),
}

impl<'a, T, M> Iterator for MultipleIdValueIter<'a, T, M> {
	type Item = TId<T>;

	fn size_hint(&self) -> (usize, Option<usize>) {
		match self {
			Self::Single(Some(_)) => (1, Some(1)),
			Self::Single(None) => (0, Some(0)),
			Self::PropertyValue(i) => i.size_hint(),
			Self::Multiple(i) => i.size_hint(),
		}
	}

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Single(i) => i.take(),
			Self::PropertyValue(i) => i.next().map(|s| **s.value),
			Self::Multiple(i) => i.next().map(|m| **m),
		}
	}
}

impl<'a, T, M> ExactSizeIterator for MultipleIdValueIter<'a, T, M> {}

impl<'a, T, M> DoubleEndedIterator for MultipleIdValueIter<'a, T, M> {
	fn next_back(&mut self) -> Option<Self::Item> {
		match self {
			Self::Single(i) => i.take(),
			Self::PropertyValue(i) => i.next_back().map(|s| **s.value),
			Self::Multiple(i) => i.next_back().map(|m| **m),
		}
	}
}

pub enum BindingValueRef<'a, M> {
	SchemaBoolean(bool),
	NonNegativeInteger(&'a NonNegativeInteger),
	String(&'a str),
	Name(&'a Name),
	Id(Id),
	Type(TId<crate::Type>),
	TypeList(MultipleIdValueRef<'a, crate::Type, M>),
	DataType(TId<crate::ty::DataType<M>>),
	Layout(TId<crate::Layout>),
	LayoutList(MultipleIdValueRef<'a, crate::Layout, M>),
	Fields(&'a [Meta<TId<layout::Field>, M>]),
	Variants(&'a [Meta<TId<layout::Variant>, M>]),
	Property(TId<crate::Property>),
	DatatypeRestrictions(ty::data::Restrictions<'a>),
	LayoutRestrictions(layout::Restrictions<'a, M>),
}

impl<'a, M> BindingValueRef<'a, M> {
	pub fn ids(self) -> BindingValueIds<'a, M> {
		match self {
			Self::Id(id) => BindingValueIds::Id(Some(id)),
			Self::Type(t) => BindingValueIds::Type(Some(t)),
			Self::TypeList(t) => BindingValueIds::TypeList(t.iter()),
			Self::DataType(t) => BindingValueIds::DataType(Some(t)),
			Self::Layout(l) => BindingValueIds::Layout(Some(l)),
			Self::LayoutList(l) => BindingValueIds::LayoutList(l.iter()),
			Self::Fields(f) => BindingValueIds::Fields(f.iter()),
			Self::Variants(v) => BindingValueIds::Variants(v.iter()),
			Self::Property(p) => BindingValueIds::Property(Some(p)),
			_ => BindingValueIds::None,
		}
	}
}

pub enum BindingValueIds<'a, M> {
	None,
	Id(Option<Id>),
	Type(Option<TId<crate::Type>>),
	TypeList(MultipleIdValueIter<'a, crate::Type, M>),
	DataType(Option<TId<crate::ty::DataType<M>>>),
	Layout(Option<TId<crate::Layout>>),
	LayoutList(MultipleIdValueIter<'a, crate::Layout, M>),
	Fields(std::slice::Iter<'a, Meta<TId<layout::Field>, M>>),
	Variants(std::slice::Iter<'a, Meta<TId<layout::Variant>, M>>),
	Property(Option<TId<crate::Property>>),
}

impl<'a, M> Iterator for BindingValueIds<'a, M> {
	type Item = Id;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::None => None,
			Self::Id(i) => i.take(),
			Self::Type(t) => t.take().map(|t| t.id()),
			Self::TypeList(t) => t.next().map(|i| i.id()),
			Self::DataType(d) => d.take().map(|i| i.id()),
			Self::Layout(l) => l.take().map(|l| l.id()),
			Self::LayoutList(t) => t.next().map(|i| i.id()),
			Self::Fields(d) => d.next().map(|i| i.id()),
			Self::Variants(t) => t.next().map(|i| i.id()),
			Self::Property(p) => p.take().map(|i| i.id()),
		}
	}
}

pub enum BindingRef<'a, M> {
	Type(Option<TId<UnknownProperty>>, TId<crate::Type>),
	Label(Option<TId<UnknownProperty>>, &'a str),
	Comment(Option<TId<UnknownProperty>>, &'a str),
	Class(crate::ty::BindingRef<'a, M>),
	Property(crate::prop::Binding),
	Component(crate::component::BindingRef<'a, M>),
}

impl<'a, M> BindingRef<'a, M> {
	pub fn domain(&self) -> Option<Type> {
		match self {
			Self::Class(b) => Some(Type::Class(b.domain())),
			Self::Property(b) => Some(Type::Property(b.domain())),
			Self::Component(b) => Some(Type::Component(b.domain())),
			_ => None,
		}
	}

	pub fn resource_property(&self) -> Property {
		match self {
			Self::Type(p, _) => Property::Type(*p),
			Self::Label(p, _) => Property::Label(*p),
			Self::Comment(p, _) => Property::Comment(*p),
			Self::Class(b) => Property::Class(b.property()),
			Self::Property(b) => Property::Property(b.property()),
			Self::Component(b) => Property::Component(b.property()),
		}
	}

	pub fn property(&self) -> crate::Property {
		crate::Property::Resource(self.resource_property())
	}

	pub fn value(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::Type(_, v) => BindingValueRef::Type(*v),
			Self::Label(_, v) => BindingValueRef::String(v),
			Self::Comment(_, v) => BindingValueRef::String(v),
			Self::Class(b) => b.value(),
			Self::Property(b) => b.value(),
			Self::Component(b) => b.value(),
		}
	}
}

/// Iterator over the bindings of a given node.
pub struct Bindings<'a, M> {
	data: ClassBindings<'a, M>,
	class: crate::ty::Bindings<'a, M>,
	property: Option<crate::prop::Bindings<'a, M>>,
	component: crate::component::Bindings<'a, M>,
}

impl<'a, M> Iterator for Bindings<'a, M> {
	type Item = Meta<BindingRef<'a, M>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.data
			.next()
			.map(|m| m.map(ClassBindingRef::into_binding_ref))
			.or_else(|| {
				self.class
					.next()
					.map(|m| m.map(BindingRef::Class))
					.or_else(|| {
						self.property
							.as_mut()
							.and_then(|i| i.next().map(|m| m.map(BindingRef::Property)))
							.or_else(|| self.component.next().map(|m| m.map(BindingRef::Component)))
					})
			})
	}
}
