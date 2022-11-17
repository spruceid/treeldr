use locspan::Meta;
use treeldr::Name;

pub use treeldr::component::{Type, Property};
use crate::{Single, layout, resource, Context, Error, error};

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

	pub fn name(&self) -> &Single<Name, M> {
		&self.data.name
	}

	pub fn name_mut(&mut self) -> &mut Single<Name, M> {
		&mut self.data.name
	}

	pub fn as_formatted(&self) -> &formatted::Definition<M> {
		&self.formatted
	}

	pub fn as_formatted_mut(&mut self) -> &mut formatted::Definition<M> {
		&mut self.formatted
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

	fn build(
		&self,
		context: &Context<M>,
		as_resource: &resource::Data<M>,
		metadata: M,
	) -> Result<treeldr::component::Definition<M>, Error<M>> {
		let name = self.data.name.try_unwrap().map_err(|e| e.at_functional_node_property(as_resource.id, Property::Name))?;

		todo!()
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

pub trait AssertNamed<M> {
	fn assert_named(&self, as_resource: &treeldr::node::Data<M>, metadata: &M) -> Result<(), Error<M>>;
}

impl<M: Clone> AssertNamed<M> for treeldr::component::Data<M> {
	fn assert_named(&self, as_resource: &treeldr::node::Data<M>, metadata: &M) -> Result<(), Error<M>> {
		self.name.ok_or_else(|| Meta(
			error::NodeBindingMissing {
				id: as_resource.id,
				property: Property::Name.into()
			}.into(),
			metadata.clone()
		))?;
		Ok(())
	}
}