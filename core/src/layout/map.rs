use locspan::Meta;

use crate::{Layout, RequiredFunctionalPropertyValue, TId};

/// Map layout.
#[derive(Debug, Clone)]
pub struct Map<M> {
	/// Key format.
	key: Meta<TId<Layout>, M>,

	/// Value format.
	value: RequiredFunctionalPropertyValue<TId<Layout>, M>,
}

impl<M> Map<M> {
	pub fn new(
		key: Meta<TId<Layout>, M>,
		value: RequiredFunctionalPropertyValue<TId<Layout>, M>,
	) -> Self {
		Self { key, value }
	}

	pub fn key_layout(&self) -> &Meta<TId<Layout>, M> {
		&self.key
	}

	pub fn value_layout(&self) -> &RequiredFunctionalPropertyValue<TId<Layout>, M> {
		&self.value
	}

	pub fn value_layout_mut(&mut self) -> &mut RequiredFunctionalPropertyValue<TId<Layout>, M> {
		&mut self.value
	}

	pub fn set_key_layout(&mut self, key: Meta<TId<Layout>, M>) {
		self.key = key
	}

	pub fn set_value_layout(&mut self, value: RequiredFunctionalPropertyValue<TId<Layout>, M>) {
		self.value = value
	}
}
