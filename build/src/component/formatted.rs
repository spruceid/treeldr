use std::cmp::Ordering;

use locspan::Meta;
use treeldr::{metadata::Merge, prop::UnknownProperty, Id, TId, Value};

use crate::{
	context::{HasType, MapIds, MapIdsIn},
	error, functional_property_value, layout,
	resource::BindingValueRef,
	Context, Error, FunctionalPropertyValue, MetaValueExt,
};

pub use treeldr::component::formatted::{Property, Type};

/// Formatted layout component.
#[derive(Clone)]
pub struct Definition<M> {
	data: Data<M>,
	layout_field: layout::field::Definition<M>,
	layout_variant: layout::variant::Definition,
}

impl<M> Default for Definition<M> {
	fn default() -> Self {
		Self {
			data: Data::default(),
			layout_field: layout::field::Definition::new(),
			layout_variant: layout::variant::Definition::new(),
		}
	}
}

impl<M> Definition<M> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn data(&self) -> &Data<M> {
		&self.data
	}

	pub fn format(&self) -> &FunctionalPropertyValue<Id, M> {
		&self.data.format
	}

	pub fn format_mut(&mut self) -> &mut FunctionalPropertyValue<Id, M> {
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

	pub fn bindings(&self) -> Bindings<M> {
		Bindings {
			data: self.data.bindings(),
			layout_field: self.layout_field.bindings(),
		}
	}

	pub fn set(
		&mut self,
		prop_cmp: impl Fn(TId<UnknownProperty>, TId<UnknownProperty>) -> Option<Ordering>,
		prop: Property,
		value: Meta<Value, M>,
	) -> Result<(), Error<M>>
	where
		M: Merge,
	{
		match prop {
			Property::Format(p) => self
				.format_mut()
				.insert(p, prop_cmp, value.into_expected_id()?),
			Property::LayoutField(prop) => self.as_layout_field_mut().set(prop_cmp, prop, value)?,
		}

		Ok(())
	}

	pub(crate) fn build(
		&self,
		context: &Context<M>,
		as_resource: &treeldr::node::Data<M>,
		as_component: &treeldr::component::Data<M>,
		meta: M,
	) -> Result<Meta<treeldr::component::formatted::Definition<M>, M>, Error<M>>
	where
		M: Clone,
	{
		let data = treeldr::component::formatted::Data {
			format: self.data.format.clone().into_layout_at_node_binding(
				context,
				as_resource.id,
				Property::Format(None),
			)?,
		};

		let layout_field = as_resource
			.type_metadata(context, Type::LayoutField)
			.map(|meta| {
				self.layout_field
					.build(context, as_resource, as_component, &data, meta.clone())
			})
			.transpose()?
			.into();

		let layout_variant = as_resource
			.type_metadata(context, Type::LayoutVariant)
			.map(|meta| {
				self.layout_variant
					.build(context, as_resource, as_component, &data, meta.clone())
			})
			.transpose()?
			.into();

		Ok(Meta(
			treeldr::component::formatted::Definition::new(data, layout_field, layout_variant),
			meta,
		))
	}
}

impl<M: Merge> MapIds for Definition<M> {
	fn map_ids(&mut self, f: impl Fn(Id, Option<crate::Property>) -> Id) {
		self.data.map_ids(&f);
		self.layout_field.map_ids(&f);
		self.layout_variant.map_ids(f)
	}
}

#[derive(Debug, Clone)]
pub struct Data<M> {
	pub format: FunctionalPropertyValue<Id, M>,
}

impl<M> Data<M> {
	pub fn bindings(&self) -> ClassBindings<M> {
		ClassBindings {
			format: self.format.iter(),
		}
	}
}

impl<M: Merge> MapIds for Data<M> {
	fn map_ids(&mut self, f: impl Fn(Id, Option<crate::Property>) -> Id) {
		self.format
			.map_ids_in(Some(Property::Format(None).into()), f)
	}
}

impl<M> Default for Data<M> {
	fn default() -> Self {
		Self {
			format: FunctionalPropertyValue::default(),
		}
	}
}

pub trait AssertFormatted<M> {
	fn assert_formatted(
		&self,
		as_resource: &treeldr::node::Data<M>,
		metadata: &M,
	) -> Result<(), Error<M>>;
}

impl<M: Clone> AssertFormatted<M> for treeldr::component::formatted::Data<M> {
	fn assert_formatted(
		&self,
		as_resource: &treeldr::node::Data<M>,
		metadata: &M,
	) -> Result<(), Error<M>> {
		self.format.as_required().ok_or_else(|| {
			Meta(
				error::NodeBindingMissing {
					id: as_resource.id,
					property: Property::Format(None).into(),
				}
				.into(),
				metadata.clone(),
			)
		})?;
		Ok(())
	}
}

pub enum ClassBinding {
	Format(Option<TId<UnknownProperty>>, Id),
}

impl ClassBinding {
	pub fn into_binding(self) -> Binding {
		match self {
			Self::Format(p, id) => Binding::Format(p, id),
		}
	}
}

#[derive(Debug)]
pub enum Binding {
	Format(Option<TId<UnknownProperty>>, Id),
	LayoutField(layout::field::ClassBinding),
}

impl Binding {
	pub fn property(&self) -> Property {
		match self {
			Self::Format(p, _) => Property::Format(*p),
			Self::LayoutField(b) => Property::LayoutField(b.property()),
		}
	}

	pub fn value<'a>(&self) -> BindingValueRef<'a> {
		match self {
			Self::Format(_, v) => BindingValueRef::Id(*v),
			Self::LayoutField(b) => b.value(),
		}
	}
}

pub struct ClassBindings<'a, M> {
	format: functional_property_value::Iter<'a, Id, M>,
}

impl<'a, M> Iterator for ClassBindings<'a, M> {
	type Item = Meta<ClassBinding, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.format
			.next()
			.map(|m| m.into_cloned_class_binding(ClassBinding::Format))
	}
}

pub struct Bindings<'a, M> {
	data: ClassBindings<'a, M>,
	layout_field: crate::layout::field::ClassBindings<'a, M>,
}

impl<'a, M> Iterator for Bindings<'a, M> {
	type Item = Meta<Binding, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.data
			.next()
			.map(|m| m.map(ClassBinding::into_binding))
			.or_else(|| {
				self.layout_field
					.next()
					.map(|m| m.map(Binding::LayoutField))
			})
	}
}
