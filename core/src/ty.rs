use crate::{
	component,
	node::{self, BindingValueRef},
	prop,
	vocab::{self, Term},
	Id, IriIndex, Model, Multiple, Ref, ResourceType, TId,
};

pub mod data;
mod intersection;
pub mod normal;
pub mod properties;
pub mod restriction;
mod r#union;

pub use data::DataType;
use derivative::Derivative;
pub use intersection::Intersection;
use locspan::Meta;
pub use normal::Normal;
pub use properties::{Properties, PseudoProperty};
pub use restriction::{Restriction, Restrictions};
pub use union::Union;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct OtherTypeId(Id);

impl OtherTypeId {
	fn id(&self) -> Id {
		self.0
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Type {
	Resource(Option<node::Type>),
	Other(OtherTypeId),
}

impl Type {
	/// Checks if this is a subclass of `other`.
	pub fn is_subclass_of(&self, other: Self) -> bool {
		match (self, other) {
			(Self::Resource(None), Self::Resource(None)) => false,
			(_, Self::Resource(None)) => true,
			(Self::Resource(Some(a)), Self::Resource(Some(b))) => a.is_subclass_of(b),
			_ => false,
		}
	}

	pub fn id(&self) -> Id {
		match self {
			Self::Resource(None) => Id::Iri(IriIndex::Iri(Term::Rdfs(vocab::Rdfs::Resource))),
			Self::Resource(Some(ty)) => Id::Iri(IriIndex::Iri(ty.term())),
			Self::Other(ty) => ty.id(),
		}
	}
}

impl ResourceType for Type {
	const TYPE: Type = Type::Resource(Some(node::Type::Class(None)));

	fn check<M>(resource: &crate::node::Definition<M>) -> bool {
		resource.is_type()
	}
}

impl<'a, M> Ref<'a, Type, M> {
	pub fn as_type(&self) -> &'a Meta<Definition<M>, M> {
		self.as_resource().as_type().unwrap()
	}
}

impl From<node::Type> for Type {
	fn from(ty: node::Type) -> Self {
		Self::Resource(Some(ty))
	}
}

impl From<SubClass> for Type {
	fn from(ty: SubClass) -> Self {
		Self::Resource(Some(node::Type::Class(Some(ty))))
	}
}

impl From<prop::Type> for Type {
	fn from(ty: prop::Type) -> Self {
		Self::Resource(Some(node::Type::Property(Some(ty))))
	}
}

impl From<component::Type> for Type {
	fn from(ty: component::Type) -> Self {
		Self::Resource(Some(node::Type::Component(Some(ty))))
	}
}

impl From<component::formatted::Type> for Type {
	fn from(ty: component::formatted::Type) -> Self {
		Self::Resource(Some(node::Type::Component(Some(
			component::Type::Formatted(Some(ty)),
		))))
	}
}

impl From<Term> for Type {
	fn from(t: Term) -> Self {
		match t {
			Term::Rdfs(vocab::Rdfs::Resource) => Self::Resource(None),
			Term::Rdfs(vocab::Rdfs::Class) => node::Type::Class(None).into(),
			Term::Rdfs(vocab::Rdfs::Datatype) => SubClass::DataType.into(),
			Term::Rdf(vocab::Rdf::Property) => node::Type::Property(None).into(),
			Term::Rdf(vocab::Rdf::List) => node::Type::List.into(),
			Term::Owl(vocab::Owl::Restriction) => SubClass::Restriction.into(),
			Term::Owl(vocab::Owl::FunctionalProperty) => prop::Type::FunctionalProperty.into(),
			Term::TreeLdr(vocab::TreeLdr::Component) => node::Type::Component(None).into(),
			Term::TreeLdr(vocab::TreeLdr::Layout) => component::Type::Layout.into(),
			Term::TreeLdr(vocab::TreeLdr::Formatted) => component::Type::Formatted(None).into(),
			Term::TreeLdr(vocab::TreeLdr::Field) => component::formatted::Type::LayoutField.into(),
			Term::TreeLdr(vocab::TreeLdr::Variant) => {
				component::formatted::Type::LayoutVariant.into()
			}
			t => Self::Other(OtherTypeId(Id::Iri(IriIndex::Iri(t)))),
		}
	}
}

impl From<Id> for Type {
	fn from(id: Id) -> Self {
		match id {
			Id::Iri(IriIndex::Iri(t)) => t.into(),
			id => Self::Other(OtherTypeId(id)),
		}
	}
}

impl From<TId<Type>> for Type {
	fn from(t: TId<Type>) -> Self {
		t.id().into()
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum SubClass {
	DataType,
	Restriction,
}

impl SubClass {
	/// Checks if this is a subclass of `other`.
	pub fn is_subclass_of(&self, _other: Self) -> bool {
		false
	}

	pub fn term(&self) -> Term {
		match self {
			Self::DataType => Term::Rdfs(vocab::Rdfs::Datatype),
			Self::Restriction => Term::Owl(vocab::Owl::Restriction),
		}
	}
}

/// Type definition.
#[derive(Debug)]
pub struct Definition<M> {
	/// Type description.
	desc: Meta<Description<M>, M>,
}

/// Type definition.
#[derive(Debug)]
pub enum Description<M> {
	Empty,
	Data(data::Definition<M>),
	Normal(Normal<M>),
	Union(Union<M>),
	Intersection(Intersection<M>),
	Restriction(restriction::Definition<M>),
}

impl<M> Description<M> {
	pub fn kind(&self) -> Kind {
		match self {
			Self::Empty => Kind::Empty,
			Self::Data(_) => Kind::Data,
			Self::Normal(_) => Kind::Normal,
			Self::Union(_) => Kind::Union,
			Self::Intersection(_) => Kind::Intersection,
			Self::Restriction(_) => Kind::Restriction,
		}
	}

	pub fn is_datatype(&self, model: &Model<M>) -> bool {
		match self {
			Self::Data(_) => true,
			Self::Union(u) => u.is_datatype(model),
			Self::Intersection(i) => i.is_datatype(model),
			_ => false,
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Kind {
	Empty,
	Data,
	Normal,
	Union,
	Intersection,
	Restriction,
}

impl<M> Definition<M> {
	pub fn new(desc: Meta<Description<M>, M>) -> Self {
		Self { desc }
	}

	pub fn description(&self) -> &Meta<Description<M>, M> {
		&self.desc
	}

	pub fn is_datatype(&self, model: &Model<M>) -> bool {
		self.desc.is_datatype(model)
	}

	pub fn union_of(&self) -> Option<Meta<&Multiple<TId<crate::Type>, M>, &M>> {
		match self.desc.value() {
			Description::Union(u) => Some(Meta(u.options(), self.desc.metadata())),
			_ => None,
		}
	}

	pub fn intersection_of(&self) -> Option<Meta<&Multiple<TId<crate::Type>, M>, &M>> {
		match self.desc.value() {
			Description::Intersection(i) => Some(Meta(i.types(), self.desc.metadata())),
			_ => None,
		}
	}

	pub fn class_bindings(&self) -> ClassBindings<M> {
		ClassBindings {
			union_of: self.union_of(),
			intersection_of: self.intersection_of(),
		}
	}

	pub fn datatype_bindings(&self) -> Option<data::Bindings<M>> {
		match self.desc.value() {
			Description::Data(dt) => Some(dt.bindings()),
			_ => None,
		}
	}

	pub fn restriction_bindings(&self) -> Option<restriction::Bindings<M>> {
		match self.desc.value() {
			Description::Restriction(r) => Some(r.bindings()),
			_ => None,
		}
	}

	pub fn bindings(&self) -> Bindings<M> {
		Bindings {
			data: self.class_bindings(),
			datatype: self.datatype_bindings().unwrap_or_default(),
			restriction: self.restriction_bindings().unwrap_or_default(),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Property {
	Datatype(data::Property),
	Restriction(restriction::Property),
	UnionOf,
	IntersectionOf,
}

impl Property {
	pub fn term(&self) -> vocab::Term {
		use vocab::Owl;
		match self {
			Self::Datatype(p) => p.term(),
			Self::Restriction(p) => p.term(),
			Self::UnionOf => Term::Owl(Owl::UnionOf),
			Self::IntersectionOf => Term::Owl(Owl::IntersectionOf),
		}
	}

	pub fn name(&self) -> &'static str {
		match self {
			Self::Datatype(p) => p.name(),
			Self::Restriction(p) => p.name(),
			Self::UnionOf => "type union",
			Self::IntersectionOf => "type intersection",
		}
	}

	pub fn expect_type(&self) -> bool {
		match self {
			Self::Datatype(p) => p.expect_type(),
			Self::Restriction(p) => p.expect_type(),
			_ => false,
		}
	}

	pub fn expect_layout(&self) -> bool {
		match self {
			Self::Datatype(p) => p.expect_layout(),
			Self::Restriction(p) => p.expect_layout(),
			_ => false,
		}
	}
}

pub enum ClassBindingRef<'a, M> {
	UnionOf(&'a Multiple<TId<crate::Type>, M>),
	IntersectionOf(&'a Multiple<TId<crate::Type>, M>),
}

impl<'a, M> ClassBindingRef<'a, M> {
	pub fn into_binding_ref(self) -> BindingRef<'a, M> {
		match self {
			Self::UnionOf(i) => BindingRef::UnionOf(i),
			Self::IntersectionOf(i) => BindingRef::IntersectionOf(i),
		}
	}
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct ClassBindings<'a, M> {
	union_of: Option<Meta<&'a Multiple<TId<crate::Type>, M>, &'a M>>,
	intersection_of: Option<Meta<&'a Multiple<TId<crate::Type>, M>, &'a M>>,
}

impl<'a, M> Iterator for ClassBindings<'a, M> {
	type Item = Meta<ClassBindingRef<'a, M>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.union_of
			.take()
			.map(|m| m.map(ClassBindingRef::UnionOf))
			.or_else(|| {
				self.intersection_of
					.take()
					.map(|m| m.map(ClassBindingRef::IntersectionOf))
			})
	}
}

pub enum BindingRef<'a, M> {
	UnionOf(&'a Multiple<TId<crate::Type>, M>),
	IntersectionOf(&'a Multiple<TId<crate::Type>, M>),
	Datatype(data::BindingRef<'a, M>),
	Restriction(restriction::Binding),
}

impl<'a, M> BindingRef<'a, M> {
	pub fn property(&self) -> Property {
		match self {
			Self::UnionOf(_) => Property::UnionOf,
			Self::IntersectionOf(_) => Property::IntersectionOf,
			Self::Datatype(b) => Property::Datatype(b.property()),
			Self::Restriction(b) => Property::Restriction(b.property()),
		}
	}

	pub fn value(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::UnionOf(v) => BindingValueRef::Types(v),
			Self::IntersectionOf(v) => BindingValueRef::Types(v),
			Self::Datatype(b) => b.value(),
			Self::Restriction(b) => b.value(),
		}
	}
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct Bindings<'a, M> {
	data: ClassBindings<'a, M>,
	datatype: data::Bindings<'a, M>,
	restriction: restriction::Bindings<'a, M>,
}

impl<'a, M> Iterator for Bindings<'a, M> {
	type Item = Meta<BindingRef<'a, M>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.data
			.next()
			.map(|m| m.map(ClassBindingRef::into_binding_ref))
			.or_else(|| {
				self.datatype
					.next()
					.map(|m| m.map(BindingRef::Datatype))
					.or_else(|| {
						self.restriction
							.next()
							.map(|m| m.map(BindingRef::Restriction))
					})
			})
	}
}
