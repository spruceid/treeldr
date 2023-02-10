use crate::{
	component, doc, error, layout, list, multiple, prop, property_values, ty,
	vocab::{self, Term},
	Documentation, Error, Id, MetaOption, Multiple, MutableModel, Name, PropertyValues,
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
			property: self
				.property
				.as_ref()
				.map(|p| p.bindings()),
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
	Self_,
	Type,
	Label,
	Comment,
	Class(ty::Property),
	DatatypeRestriction(ty::data::restriction::Property),
	Property(prop::RdfProperty),
	Component(component::Property),
	LayoutRestriction(layout::restriction::Property),
	List(list::Property),
}

impl Property {
	pub fn term(&self) -> vocab::Term {
		use vocab::{Rdf, Rdfs};
		match self {
			Self::Self_ => Term::TreeLdr(vocab::TreeLdr::Self_),
			Self::Type => Term::Rdf(Rdf::Type),
			Self::Label => Term::Rdfs(Rdfs::Label),
			Self::Comment => Term::Rdfs(Rdfs::Comment),
			Self::Class(p) => p.term(),
			Self::DatatypeRestriction(p) => p.term(),
			Self::Property(p) => p.term(),
			Self::Component(p) => p.term(),
			Self::LayoutRestriction(p) => p.term(),
			Self::List(p) => p.term(),
		}
	}

	pub fn name(&self) -> &'static str {
		match self {
			Self::Self_ => "self reference",
			Self::Type => "type",
			Self::Label => "label",
			Self::Comment => "comment",
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
			Self::Type => true,
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
	Type(Option<Id>, TId<crate::Type>),
	Label(Option<Id>, &'a str),
	Comment(Option<Id>, &'a str),
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
	Types(MultipleIdValueRef<'a, crate::Type, M>),
	DataType(TId<crate::ty::DataType<M>>),
	Layouts(MultipleIdValueRef<'a, crate::Layout, M>),
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
			Self::Types(t) => BindingValueIds::Types(t.iter()),
			Self::DataType(t) => BindingValueIds::DataType(Some(t)),
			Self::Layouts(l) => BindingValueIds::Layouts(l.iter()),
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
	Types(MultipleIdValueIter<'a, crate::Type, M>),
	DataType(Option<TId<crate::ty::DataType<M>>>),
	Layouts(MultipleIdValueIter<'a, crate::Layout, M>),
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
			Self::Types(t) => t.next().map(|i| i.id()),
			Self::DataType(d) => d.take().map(|i| i.id()),
			Self::Layouts(t) => t.next().map(|i| i.id()),
			Self::Fields(d) => d.next().map(|i| i.id()),
			Self::Variants(t) => t.next().map(|i| i.id()),
			Self::Property(p) => p.take().map(|i| i.id()),
		}
	}
}

pub enum BindingRef<'a, M> {
	Type(Option<Id>, TId<crate::Type>),
	Label(Option<Id>, &'a str),
	Comment(Option<Id>, &'a str),
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
			Self::Type(_, _) => Property::Type,
			Self::Label(_, _) => Property::Label,
			Self::Comment(_, _) => Property::Comment,
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
			Self::Type(_, v) => BindingValueRef::Types(MultipleIdValueRef::Single(*v)),
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
							.and_then(|i| {
								i
									.next()
									.map(|m| m.map(BindingRef::Property))
							})
							.or_else(|| self.component.next().map(|m| m.map(BindingRef::Component)))
					})
			})
	}
}
