use crate::TId;
use locspan::Meta;

use super::field::Field;

/// Structure layout.
#[derive(Debug, Clone)]
pub struct Struct<M> {
	/// List of fields.
	fields: Vec<Meta<TId<Field>, M>>,
}

impl<M> Struct<M> {
	pub fn new(fields: Vec<Meta<TId<Field>, M>>) -> Self {
		Self { fields }
	}

	pub fn fields(&self) -> &[Meta<TId<Field>, M>] {
		&self.fields
	}

	pub fn fields_mut(&mut self) -> &mut [Meta<TId<Field>, M>] {
		&mut self.fields
	}

	// pub fn as_sum_option(&self) -> Option<TId<Layout>> {
	// 	if self.fields.len() == 1 {
	// 		Some(self.fields[0].layout())
	// 	} else {
	// 		None
	// 	}
	// }
}
