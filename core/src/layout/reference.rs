use locspan::Meta;

use crate::{TId, Layout};

/// Reference layout.
#[derive(Debug, Clone)]
pub struct Reference<M> {
	/// Layout used to store the id of the referenced resource.
	id_layout: Meta<TId<Layout>, M>,
}

impl<M> Reference<M> {
	pub fn new(id_layout: Meta<TId<Layout>, M>) -> Self {
		Self { id_layout }
	}

	pub fn id_layout(&self) -> &Meta<TId<Layout>, M> {
		&self.id_layout
	}

	pub fn set_id_layout(&mut self, id_layout: Meta<TId<Layout>, M>) {
		self.id_layout = id_layout
	}
}