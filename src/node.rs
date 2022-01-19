use crate::{
	Model,
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
	Layout
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Node {
	Type(Ref<ty::Definition>),
	Property(Ref<prop::Definition>),
	Layout(Ref<layout::Definition>),
	Unknown(Id)
}

impl Node {
	pub fn ty(&self) -> Option<Type> {
		match self {
			Self::Type(_) => Some(Type::Type),
			Self::Property(_) => Some(Type::Property),
			Self::Layout(_) => Some(Type::Layout),
			Self::Unknown(_) => None
		}
	}

	pub fn id(&self, context: &Model) -> Id {
		match self {
			Self::Type(r) => context.types().get(*r).expect("undefined type").id(),
			Self::Property(r) => context.properties().get(*r).expect("undefined property").id(),
			Self::Layout(r) => context.layouts().get(*r).expect("undefined layout").id(),
			Self::Unknown(id) => *id
		}
	}

	pub fn as_type(&self) -> Option<Ref<ty::Definition>> {
		match self {
			Self::Type(ty_ref) => Some(*ty_ref),
			_ => None
		}
	}

	pub fn as_property(&self) -> Option<Ref<prop::Definition>> {
		match self {
			Self::Property(prop_ref) => Some(*prop_ref),
			_ => None
		}
	}

	pub fn as_layout(&self) -> Option<Ref<layout::Definition>> {
		match self {
			Self::Layout(layout_ref) => Some(*layout_ref),
			_ => None
		}
	}
}