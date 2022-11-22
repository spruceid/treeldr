use locspan::Meta;
use treeldr::{Name, metadata::Merge};

pub use treeldr::component::{Type, Property};
use crate::{Single, layout, Context, Error, error, context::HasType, single};

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

	pub fn data(&self) -> &Data<M> {
		&self.data
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

	pub(crate) fn build(
		&self,
		context: &Context<M>,
		as_resource: &treeldr::node::Data<M>,
		metadata: M,
	) -> Result<Meta<treeldr::component::Definition<M>, M>, Error<M>> where M: Clone + Merge {
		let data = treeldr::component::Data {
			name: self.data.name.clone().try_unwrap().map_err(|e| e.at_functional_node_property(as_resource.id, Property::Name))?
		};

		let layout = as_resource.type_metadata(context, Type::Layout).map(|meta| {
			self.layout.build(context, as_resource, &data, meta.clone())
		}).transpose()?.into();

		let formatted = as_resource.type_metadata(context, Type::Formatted(None)).map(|meta| {
			self.formatted.build(context, as_resource, &data, meta.clone())
		}).transpose()?.into();

		Ok(Meta(treeldr::component::Definition::new(data, layout, formatted), metadata))
	}
}

#[derive(Debug, Clone)]
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
		self.name.as_ref().ok_or_else(|| Meta(
			error::NodeBindingMissing {
				id: as_resource.id,
				property: Property::Name.into()
			}.into(),
			metadata.clone()
		))?;
		Ok(())
	}
}

pub enum ClassBindingRef<'a> {
	Name(&'a Name)
}

impl<'a> ClassBindingRef<'a> {
	pub fn into_binding_ref(self) -> BindingRef<'a> {
		match self {
			Self::Name(n) => BindingRef::Name(n)
		}
	}
}

pub enum BindingRef<'a> {
	Name(&'a Name),
	Layout(layout::Binding),
	Formatted(formatted::Binding)
}

pub struct ClassBindings<'a, M> {
	name: single::Iter<'a, Name, M>
}

impl<'a, M> Iterator for ClassBindings<'a, M> {
	type Item = Meta<ClassBindingRef<'a>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.name.next().map(|m| m.map(ClassBindingRef::Name))
	}
}

pub struct Bindings<'a, M> {
	data: ClassBindings<'a, M>,
	layout: layout::Bindings<'a, M>,
	formatted: formatted::Bindings<'a, M>
}

impl<'a, M> Iterator for Bindings<'a, M> {
	type Item = Meta<BindingRef<'a>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.data
			.next()
			.map(|m| m.map(ClassBindingRef::into_binding_ref))
			.or_else(|| {
				self.layout
					.next()
					.map(|m| m.map(BindingRef::Layout))
					.or_else(|| {
						self.formatted
							.next()
							.map(|m| m.map(BindingRef::Formatted))
					})
			})
	}
}