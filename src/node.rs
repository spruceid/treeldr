use crate::{
	Context,
	Id,
	Ref,
	ty,
	prop,
	layout
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Type {
	Type,
	Property,
	Layout,
	Unknown
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Node {
	Type(Ref<ty::Definition>),
	Property(Ref<prop::Definition>),
	Layout(Ref<layout::Definition>),
	Unknown(Id)
}

impl Node {
	pub fn ty(&self) -> Type {
		match self {
			Self::Type(_) => Type::Type,
			Self::Property(_) => Type::Property,
			Self::Layout(_) => Type::Layout,
			Self::Unknown(_) => Type::Unknown
		}
	}

	pub fn id(&self, context: &Context) -> Id {
		match self {
			Self::Type(r) => context.types().get(*r).expect("undefined type").id(),
			Self::Property(r) => context.properties().get(*r).expect("undefined property").id(),
			Self::Layout(r) => context.layouts().get(*r).expect("undefined layout").id(),
			Self::Unknown(id) => *id
		}
	}
}