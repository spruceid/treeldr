use super::ContainerRestrictions;
use crate::{Layout, MetaOption, TId};
use locspan::Meta;

/// Set layout.
#[derive(Debug, Clone)]
pub struct Set<M> {
	/// Item layout.
	item: Meta<TId<Layout>, M>,

	/// Restrictions.
	restrictions: MetaOption<ContainerRestrictions<M>, M>,
}

impl<M> Set<M> {
	pub fn new(
		item: Meta<TId<Layout>, M>,
		restrictions: MetaOption<ContainerRestrictions<M>, M>,
	) -> Self {
		Self { item, restrictions }
	}

	pub fn item_layout(&self) -> &Meta<TId<Layout>, M> {
		&self.item
	}

	pub fn set_item_layout(&mut self, item: Meta<TId<Layout>, M>) {
		self.item = item
	}

	pub fn restrictions(&self) -> &MetaOption<ContainerRestrictions<M>, M> {
		&self.restrictions
	}

	pub fn is_required(&self) -> bool {
		self.restrictions
			.is_some_and(ContainerRestrictions::is_required)
	}
}
