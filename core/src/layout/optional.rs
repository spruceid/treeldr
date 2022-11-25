use crate::{Layout, TId};
use locspan::Meta;

#[derive(Debug, Clone)]
pub struct Optional<M> {
	/// Item layout.
	item: Meta<TId<Layout>, M>,
}

impl<M> Optional<M> {
	pub fn new(item: Meta<TId<Layout>, M>) -> Self {
		Self { item }
	}

	pub fn item_layout(&self) -> &Meta<TId<Layout>, M> {
		&self.item
	}

	pub fn set_item_layout(&mut self, item: Meta<TId<Layout>, M>) {
		self.item = item
	}
}
