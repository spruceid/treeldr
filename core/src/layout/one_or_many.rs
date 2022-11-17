use super::Restrictions;
use crate::{TId, Layout};
use locspan::Meta;

/// "One or many" layout.
#[derive(Debug, Clone)]
pub struct OneOrMany<M> {
	/// Item layout.
	item: Meta<TId<Layout>, M>,

	/// Restrictions.
	restrictions: Restrictions<M>,
}

impl<M> OneOrMany<M> {
	pub fn new(
		item: Meta<TId<Layout>, M>,
		restrictions: Restrictions<M>,
	) -> Self {
		Self {
			item,
			restrictions,
		}
	}

	pub fn item_layout(&self) -> &Meta<TId<Layout>, M> {
		&self.item
	}

	pub fn set_item_layout(&mut self, item: Meta<TId<Layout>, M>) {
		self.item = item
	}

	pub fn restrictions(&self) -> &Restrictions<M> {
		&self.restrictions
	}

	pub fn is_required(&self) -> bool {
		self.restrictions.is_required()
	}
}