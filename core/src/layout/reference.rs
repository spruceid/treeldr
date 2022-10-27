use crate::{Id, MetaOption, Name, Ref, SubstituteReferences};
use locspan::Meta;

/// Reference layout.
#[derive(Clone)]
pub struct Reference<M> {
	/// Optional layout name.
	name: MetaOption<Name, M>,

	/// Layout used to store the id of the referenced resource.
	id_layout: Ref<super::Definition<M>>,
}

impl<M> Reference<M> {
	pub fn new(name: MetaOption<Name, M>, id_layout: Ref<super::Definition<M>>) -> Self {
		Self { name, id_layout }
	}

	pub fn name(&self) -> Option<&Meta<Name, M>> {
		self.name.as_ref()
	}

	pub fn set_name(&mut self, new_name: Name, metadata: M) -> Option<Meta<Name, M>> {
		self.name.replace(new_name, metadata)
	}

	pub fn into_name(self) -> MetaOption<Name, M> {
		self.name
	}

	pub fn id_layout(&self) -> Ref<super::Definition<M>> {
		self.id_layout
	}

	pub fn set_id_layout(&mut self, id_layout: Ref<super::Definition<M>>) {
		self.id_layout = id_layout
	}
}

impl<M> SubstituteReferences<M> for Reference<M> {
	fn substitute_references<I, T, P, L>(&mut self, sub: &crate::ReferenceSubstitution<I, T, P, L>)
	where
		I: Fn(Id) -> Id,
		T: Fn(Ref<crate::ty::Definition<M>>) -> Ref<crate::ty::Definition<M>>,
		P: Fn(Ref<crate::prop::Definition<M>>) -> Ref<crate::prop::Definition<M>>,
		L: Fn(Ref<super::Definition<M>>) -> Ref<super::Definition<M>>,
	{
		self.id_layout = sub.layout(self.id_layout)
	}
}
