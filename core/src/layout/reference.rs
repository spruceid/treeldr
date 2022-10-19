use crate::{MetaOption, Name, Ref};
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
