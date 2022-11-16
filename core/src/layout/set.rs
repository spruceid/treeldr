use super::Restrictions;
use crate::{MetaOption, Name, Layout, TId};
use locspan::Meta;

/// Set layout.
#[derive(Debug, Clone)]
pub struct Set<M> {
	/// Layout name, if any.
	name: MetaOption<Name, M>,

	/// Item layout.
	item: TId<Layout>,

	/// Restrictions.
	restrictions: Restrictions<M>,
}

impl<M> Set<M> {
	pub fn new(
		name: MetaOption<Name, M>,
		item: TId<Layout>,
		restrictions: Restrictions<M>,
	) -> Self {
		Self {
			name,
			item,
			restrictions,
		}
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

	pub fn item_layout(&self) -> TId<Layout> {
		self.item
	}

	pub fn set_item_layout(&mut self, item: TId<Layout>) {
		self.item = item
	}

	pub fn restrictions(&self) -> &Restrictions<M> {
		&self.restrictions
	}

	pub fn is_required(&self) -> bool {
		self.restrictions.is_required()
	}
}