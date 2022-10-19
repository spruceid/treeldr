use crate::{MetaOption, Name, Ref};
use locspan::Meta;

#[derive(Clone)]
pub struct Optional<M> {
	/// Layout name, if any.
	name: MetaOption<Name, M>,

	/// Item layout.
	item: Ref<super::Definition<M>>,
}

impl<M> Optional<M> {
	pub fn new(name: MetaOption<Name, M>, item: Ref<super::Definition<M>>) -> Self {
		Self { name, item }
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

	pub fn item_layout(&self) -> Ref<super::Definition<M>> {
		self.item
	}

	pub fn set_item_layout(&mut self, item: Ref<super::Definition<M>>) {
		self.item = item
	}
}
