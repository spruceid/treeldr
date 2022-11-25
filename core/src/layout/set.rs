use super::ContainerRestrictions;
use crate::{Layout, TId};
use locspan::Meta;

/// Set layout.
#[derive(Debug, Clone)]
pub struct Set<M> {
	/// Item layout.
	item: Meta<TId<Layout>, M>,

	/// Restrictions.
	restrictions: ContainerRestrictions<M>,
}

impl<M> Set<M> {
	pub fn new(item: Meta<TId<Layout>, M>, restrictions: ContainerRestrictions<M>) -> Self {
		Self { item, restrictions }
	}

	pub fn item_layout(&self) -> &Meta<TId<Layout>, M> {
		&self.item
	}

	pub fn set_item_layout(&mut self, item: Meta<TId<Layout>, M>) {
		self.item = item
	}

	pub fn restrictions(&self) -> &ContainerRestrictions<M> {
		&self.restrictions
	}

	pub fn is_required(&self) -> bool {
		self.restrictions.is_required()
	}
}
