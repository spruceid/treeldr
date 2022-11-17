use locspan::Meta;
use treeldr::Id;

use crate::{Single, layout, Error, error};

pub use treeldr::component::formatted::{Type, Property};

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

	pub fn format(&self) -> &Single<Id, M> {
		&self.data.format
	}

	pub fn format_mut(&mut self) -> &mut Single<Id, M> {
		&mut self.data.format
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

pub trait AssertFormatted<M> {
	fn assert_formatted(&self, as_resource: &treeldr::node::Data<M>, metadata: &M) -> Result<(), Error<M>>;
}

impl<M: Clone> AssertFormatted<M> for treeldr::component::formatted::Data<M> {
	fn assert_formatted(&self, as_resource: &treeldr::node::Data<M>, metadata: &M) -> Result<(), Error<M>> {
		self.format.ok_or_else(|| Meta(
			error::NodeBindingMissing {
				id: as_resource.id,
				property: Property::Name.into()
			}.into(),
			metadata.clone()
		))?;
		Ok(())
	}
}