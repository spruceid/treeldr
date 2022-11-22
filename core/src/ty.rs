use crate::{Model, component, ResourceType, Ref, vocab::{self, Term}, prop, Id, node, IriIndex};

pub mod data;
mod intersection;
pub mod normal;
pub mod properties;
pub mod restriction;
mod r#union;

pub use data::DataType;
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
	Other(OtherTypeId)
}

impl Type {
	/// Checks if this is a subclass of `other`.
	pub fn is_subclass_of(&self, other: Self) -> bool {
		match (self, other) {
			(Self::Resource(None), Self::Resource(None)) => false,
			(_, Self::Resource(None)) => true,
			(Self::Resource(Some(a)), Self::Resource(Some(b))) => a.is_subclass_of(b),
			_ => false
		}
	}

	pub fn id(&self) -> Id {
		match self {
			Self::Resource(None) => Id::Iri(IriIndex::Iri(Term::Rdfs(vocab::Rdfs::Resource))),
			Self::Resource(Some(ty)) => Id::Iri(IriIndex::Iri(ty.term())),
			Self::Other(ty) => ty.id()
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
		Self::Resource(Some(node::Type::Component(Some(component::Type::Formatted(Some(ty))))))
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum SubClass {
	DataType,
	Restriction
}

impl SubClass {
	/// Checks if this is a subclass of `other`.
	pub fn is_subclass_of(&self, _other: Self) -> bool {
		false
	}

	pub fn term(&self) -> Term {
		match self {
			Self::DataType => Term::Rdfs(vocab::Rdfs::Datatype),
			Self::Restriction => Term::Owl(vocab::Owl::Restriction)
		}
	}
}

/// Type definition.
#[derive(Debug)]
pub struct Definition<M> {
	/// Type description.
	desc: Description<M>,
}

/// Type definition.
#[derive(Debug)]
pub enum Description<M> {
	Empty,
	Data(data::Definition),
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
	pub fn new(desc: Description<M>) -> Self {
		Self {
			desc,
		}
	}

	pub fn description(&self) -> &Description<M> {
		&self.desc
	}

	pub fn is_datatype(&self, model: &Model<M>) -> bool {
		self.desc.is_datatype(model)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Property {
	Datatype(data::Property),
	Restriction(restriction::Property),
	UnionOf,
	IntersectionOf
}

impl Property {
	pub fn term(&self) -> vocab::Term {
		use vocab::Owl;
		match self {
			Self::Datatype(p) => p.term(),
			Self::Restriction(p) => p.term(),
			Self::UnionOf => Term::Owl(Owl::UnionOf),
			Self::IntersectionOf => Term::Owl(Owl::IntersectionOf)
		}
	}

	pub fn name(&self) -> &'static str {
		match self {
			Self::Datatype(p) => p.name(),
			Self::Restriction(p) => p.name(),
			Self::UnionOf => "type union",
			Self::IntersectionOf => "type intersection"
		}
	}
}