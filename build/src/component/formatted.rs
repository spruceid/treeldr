use treeldr::Id;

use crate::{Single, layout};

/// Formatted layout component.
#[derive(Clone)]
pub struct Definition<M> {
	data: Data<M>,
	layout_field: layout::field::Definition<M>,
	layout_variant: layout::variant::Definition,
}

impl<M> Definition<M> {
	pub fn new() -> Self {
		Self {
			data: Data::default(),
			layout_field: layout::field::Definition::new(),
			layout_variant: layout::variant::Definition::new()
		}
	}

	pub fn as_layout_field(&self) -> &layout::field::Definition<M> {
		&self.layout_field
	}

	pub fn as_layout_field_mut(&mut self) -> &mut layout::field::Definition<M> {
		&mut self.layout_field
	}

	pub fn as_layout_variant(&self) -> &layout::variant::Definition {
		&self.layout_variant
	}

	pub fn as_layout_variant_mut(&mut self) -> &mut layout::variant::Definition {
		&mut self.layout_variant
	}
}

#[derive(Clone)]
pub struct Data<M> {
	pub format: Single<Id, M>
}

impl<M> Default for Data<M> {
	fn default() -> Self {
		Self {
			format: Single::default()
		}
	}
}