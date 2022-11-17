use crate::{Model, component, ResourceType, Ref, vocab};

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
pub enum Type {
	Resource,
	Class(Option<SubClass>),
	DatatypeRestriction,
	Property,
	Component(Option<component::Type>),
	LayoutRestriction,
	List,
}

impl From<SubClass> for Type {
	fn from(ty: SubClass) -> Self {
		Self::Class(Some(ty))
	}
}

impl From<component::Type> for Type {
	fn from(ty: component::Type) -> Self {
		Self::Component(Some(ty))
	}
}

impl From<component::formatted::Type> for Type {
	fn from(ty: component::formatted::Type) -> Self {
		Self::Component(Some(component::Type::Formatted(Some(ty))))
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum SubClass {
	DataType,
	Restriction
}

impl ResourceType for Type {
	const TYPE: Type = Type::Class(None);

	fn check<M>(resource: &crate::node::Definition<M>) -> bool {
		resource.is_type()
	}
}

impl<'a, M> Ref<'a, Type, M> {
	pub fn as_type(&self) -> &'a Meta<Definition<M>, M> {
		self.as_resource().as_type().unwrap()
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
		use vocab::{Term, Owl};
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