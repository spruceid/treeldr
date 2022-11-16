use crate::{
	MetaOption, Name, TId, Layout
};
use locspan::Meta;

/// Structure layout.
#[derive(Debug, Clone)]
pub struct Struct<M> {
	name: Meta<Name, M>,
	fields: Vec<Field<M>>,
}

/// Structure layout parts.
#[derive(Clone)]
pub struct Parts<M> {
	pub name: Meta<Name, M>,
	pub fields: Vec<Field<M>>,
}

impl<M> Struct<M> {
	pub fn new(name: Meta<Name, M>, fields: Vec<Field<M>>) -> Self {
		Self { name, fields }
	}

	pub fn into_parts(self) -> Parts<M> {
		Parts {
			name: self.name,
			fields: self.fields,
		}
	}

	pub fn name(&self) -> &Meta<Name, M> {
		&self.name
	}

	pub fn into_name(self) -> Meta<Name, M> {
		self.name
	}

	pub fn set_name(&mut self, new_name: Name, metadata: M) -> Meta<Name, M> {
		std::mem::replace(&mut self.name, Meta::new(new_name, metadata))
	}

	pub fn fields(&self) -> &[Field<M>] {
		&self.fields
	}

	pub fn fields_mut(&mut self) -> &mut [Field<M>] {
		&mut self.fields
	}

	pub fn as_sum_option(&self) -> Option<TId<Layout>> {
		if self.fields.len() == 1 {
			Some(self.fields[0].layout())
		} else {
			None
		}
	}
}

/// Layout field.
#[derive(Debug, Clone)]
pub struct Field<M> {
	prop: MetaOption<TId<crate::Property>, M>,
	layout: Meta<TId<Layout>, M>
}

/// Layout field parts.
#[derive(Clone)]
pub struct FieldsParts<M> {
	pub prop: MetaOption<TId<crate::Property>, M>,
	pub layout: Meta<TId<Layout>, M>
}

impl<M> Field<M> {
	pub fn new(
		prop: MetaOption<TId<crate::Property>, M>,
		layout: Meta<TId<Layout>, M>
	) -> Self {
		Self {
			prop,
			layout,
		}
	}

	pub fn into_parts(self) -> FieldsParts<M> {
		FieldsParts {
			prop: self.prop,
			layout: self.layout
		}
	}

	pub fn property(&self) -> Option<TId<crate::Property>> {
		self.prop.value().cloned()
	}

	pub fn property_with_causes(&self) -> &MetaOption<TId<crate::Property>, M> {
		&self.prop
	}

	pub fn layout(&self) -> TId<Layout> {
		*self.layout
	}

	pub fn layout_with_causes(&self) -> &Meta<TId<Layout>, M> {
		&self.layout
	}

	pub fn is_required(&self, model: &crate::Model<M>) -> bool {
		let layout = model.get(self.layout()).unwrap().as_layout();
		layout.is_required()
	}

	pub fn set_layout(&mut self, layout: TId<Layout>, metadata: M) {
		self.layout = Meta::new(layout, metadata)
	}
}