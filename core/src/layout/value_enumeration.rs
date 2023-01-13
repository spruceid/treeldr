use crate::TId;
use locspan::Meta;

pub mod variant;

pub use variant::ValueVariant;

/// Enumeration.
#[derive(Debug, Clone)]
pub struct ValueEnum<M> {
	members: Vec<Meta<TId<ValueVariant>, M>>,
}

pub struct Parts<M> {
	pub members: Vec<Meta<TId<ValueVariant>, M>>,
}

impl<M> ValueEnum<M> {
	pub fn new(members: Vec<Meta<TId<ValueVariant>, M>>) -> Self {
		Self { members }
	}

	pub fn members(&self) -> &[Meta<TId<ValueVariant>, M>] {
		&self.members
	}
}