use crate::{TId, Layout};
use locspan::Meta;

#[derive(Debug, Clone)]
pub struct Required<M> {
	/// Item layout.
	item: Meta<TId<Layout>, M>,
}

impl<M> Required<M> {
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