use treeldr::Name;

pub use treeldr::component::Type;
use crate::{Single, layout};

pub mod formatted;

/// Layout component.
#[derive(Clone)]
pub struct Definition<M> {
	data: Data<M>,
	layout: layout::Definition<M>,
	formatted: formatted::Definition<M>
}

impl<M> Definition<M> {
	pub fn new() -> Self {
		Self {
			data: Data::default(),
			layout: layout::Definition::new(),
			formatted: formatted::Definition::new()
		}
	}

	pub fn as_layout(&self) -> &layout::Definition<M> {
		&self.layout
	}

	pub fn as_layout_mut(&mut self) -> &mut layout::Definition<M> {
		&mut self.layout
	}

	pub fn as_layout_field(&self) -> &layout::field::Definition<M> {
		self.formatted.as_layout_field()
	}

	pub fn as_layout_field_mut(&mut self) -> &mut layout::field::Definition<M> {
		self.formatted.as_layout_field_mut()
	}

	pub fn as_layout_variant(&self) -> &layout::variant::Definition {
		self.formatted.as_layout_variant()
	}

	pub fn as_layout_variant_mut(&mut self) -> &mut layout::variant::Definition {
		self.formatted.as_layout_variant_mut()
	}
}

#[derive(Clone)]
pub struct Data<M> {
	pub name: Single<Name, M>
}

impl<M> Default for Data<M> {
	fn default() -> Self {
		Self {
			name: Single::default()
		}
	}
}