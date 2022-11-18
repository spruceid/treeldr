use locspan::Meta;
use treeldr::Id;

use crate::{Single, layout, Error, error, Context, context::HasType};

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

	pub fn data(&self) -> &Data<M> {
		&self.data
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

	pub(crate) fn build(
		&self,
		context: &Context<M>,
		as_resource: &treeldr::node::Data<M>,
		as_component: &treeldr::component::Data<M>,
		meta: M
	) -> Result<Meta<treeldr::component::formatted::Definition<M>, M>, Error<M>> where M: Clone {
		let data = treeldr::component::formatted::Data {
			format: self.data.format.clone().into_layout_at_node_binding(context, as_resource.id, Property::Format)?
		};

		let layout_field = as_resource.type_metadata(context, Type::LayoutField).map(|meta| {
			self.layout_field.build(context, as_resource, as_component, &data, meta.clone())
		}).transpose()?.into();

		let layout_variant = as_resource.type_metadata(context, Type::LayoutVariant).map(|meta| {
			self.layout_variant.build(context, as_resource, as_component, &data, meta.clone())
		}).transpose()?.into();

		Ok(Meta(treeldr::component::formatted::Definition::new(data, layout_field, layout_variant), meta))
	}
}

#[derive(Debug, Clone)]
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
		self.format.as_ref().ok_or_else(|| Meta(
			error::NodeBindingMissing {
				id: as_resource.id,
				property: Property::Format.into()
			}.into(),
			metadata.clone()
		))?;
		Ok(())
	}
}