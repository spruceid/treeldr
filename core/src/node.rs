use crate::{
	component, doc, error, layout, list, multiple, prop, ty,
	vocab::{self, Term},
	Documentation, Error, Id, MetaOption, Model, Multiple, Name, ResourceType, TId,
};
use locspan::Meta;

#[derive(Debug, Clone)]
pub struct Data<M> {
	pub id: Id,
	pub metadata: M,
	pub type_: Multiple<TId<crate::Type>, M>,
	pub label: Multiple<String, M>,
	pub comment: Documentation<M>,
}

impl<M> Data<M> {
	pub fn new(id: Id, metadata: M) -> Self {
		Self {
			id,
			metadata,
			type_: Multiple::default(),
			label: Multiple::default(),
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

	pub fn type_(&self) -> &Multiple<TId<crate::Type>, M> {
		&self.data.type_
	}

	pub fn label(&self) -> &Multiple<String, M> {
		&self.data.label
	}

	pub fn preferred_label(&self) -> Option<&str> {
		self.data.label.first().map(|m| m.value().as_str())
	}

	pub fn comment(&self) -> &Documentation<M> {
		&self.data.comment
	}

	pub fn is_type(&self) -> bool {
		self.ty.is_some()
	}

	pub fn is_datatype(&self, model: &Model<M>) -> bool {
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
				expected: TId::new(
					crate::Type::Resource(Some(Type::Component(Some(component::Type::Layout))))
						.id(),
				),
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
				.map(|p| p.bindings())
				.unwrap_or_default(),
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
	Type(TId<crate::Type>),
	Label(&'a str),
	Comment(&'a str),
}

impl<'a> ClassBindingRef<'a> {
	pub fn into_binding_ref<M>(self) -> BindingRef<'a, M> {
		match self {
			Self::Type(t) => BindingRef::Type(t),
			Self::Label(l) => BindingRef::Label(l),
			Self::Comment(c) => BindingRef::Comment(c),
		}
	}
}

pub struct ClassBindings<'a, M> {
	type_: multiple::Iter<'a, TId<crate::Type>, M>,
	label: multiple::Iter<'a, String, M>,
	comment: doc::Iter<'a, M>,
}

impl<'a, M> Iterator for ClassBindings<'a, M> {
	type Item = Meta<ClassBindingRef<'a>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.type_
			.next()
			.map(Meta::into_cloned_value)
			.map(|m| m.map(ClassBindingRef::Type))
			.or_else(|| {
				self.label
					.next()
					.map(|v| v.map(String::as_str))
					.map(|m| m.map(ClassBindingRef::Label))
					.or_else(|| {
						self.comment
							.next()
							.map(|v| v.map(doc::Block::as_str))
							.map(|m| m.map(ClassBindingRef::Comment))
					})
			})
	}
}

pub enum BindingValueRef<'a, M> {
	Boolean(bool),
	U64(u64),
	String(&'a str),
	Name(&'a Name),
	Id(Id),
	Type(TId<crate::Type>),
	Types(&'a Multiple<TId<crate::Type>, M>),
	DataType(TId<crate::ty::DataType<M>>),
	Layout(TId<crate::Layout>),
	Layouts(&'a Multiple<TId<crate::Layout>, M>),
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
			Self::Types(t) => BindingValueIds::Types(t.iter()),
			Self::DataType(t) => BindingValueIds::DataType(Some(t)),
			Self::Layout(l) => BindingValueIds::Layout(Some(l)),
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
	Type(Option<TId<crate::Type>>),
	Types(multiple::Iter<'a, TId<crate::Type>, M>),
	DataType(Option<TId<crate::ty::DataType<M>>>),
	Layout(Option<TId<crate::Layout>>),
	Layouts(multiple::Iter<'a, TId<crate::Layout>, M>),
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
			Self::Type(t) => t.take().map(|i| i.id()),
			Self::Types(t) => t.next().map(|i| i.id()),
			Self::DataType(d) => d.take().map(|i| i.id()),
			Self::Layout(t) => t.take().map(|i| i.id()),
			Self::Layouts(t) => t.next().map(|i| i.id()),
			Self::Fields(d) => d.next().map(|i| i.id()),
			Self::Variants(t) => t.next().map(|i| i.id()),
			Self::Property(p) => p.take().map(|i| i.id()),
		}
	}
}

pub enum BindingRef<'a, M> {
	Type(TId<crate::Type>),
	Label(&'a str),
	Comment(&'a str),
	Class(crate::ty::BindingRef<'a, M>),
	Property(crate::prop::Binding),
	Component(crate::component::BindingRef<'a, M>),
}

impl<'a, M> BindingRef<'a, M> {
	pub fn resource_property(&self) -> Property {
		match self {
			Self::Type(_) => Property::Type,
			Self::Label(_) => Property::Label,
			Self::Comment(_) => Property::Comment,
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
			Self::Type(v) => BindingValueRef::Type(*v),
			Self::Label(v) => BindingValueRef::String(v),
			Self::Comment(v) => BindingValueRef::String(v),
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
	property: crate::prop::Bindings<'a, M>,
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
							.next()
							.map(|m| m.map(BindingRef::Property))
							.or_else(|| self.component.next().map(|m| m.map(BindingRef::Component)))
					})
			})
	}
}
