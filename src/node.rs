use crate::{
	Context,
	Id,
	Ref,
	ty,
	prop,
	layout
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Node {
	Type(Ref<ty::Definition>),
	Property(Ref<prop::Definition>),
	Layout(Ref<layout::Definition>),
	Unknown(Id)
}

impl Node {
	pub fn id(&self, context: &Context) -> Id {
		match self {
			Self::Type(r) => context.types().get(*r).expect("undefined type").id(),
			Self::Property(r) => context.properties().get(*r).expect("undefined property").id(),
			Self::Layout(r) => context.layouts().get(*r).expect("undefined layout").id(),
			Self::Unknown(id) => *id
		}
	}
}