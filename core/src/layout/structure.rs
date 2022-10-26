use crate::{layout, prop, Documentation, MetaOption, Name};
use locspan::Meta;
use shelves::Ref;

/// Structure layout.
#[derive(Clone)]
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

	pub fn as_sum_option(&self) -> Option<Ref<super::Definition<M>>> {
		if self.fields.len() == 1 {
			Some(self.fields[0].layout())
		} else {
			None
		}
	}
}

/// Layout field.
#[derive(Clone)]
pub struct Field<M> {
	prop: MetaOption<Ref<prop::Definition<M>>, M>,
	name: Meta<Name, M>,
	label: Option<String>,
	layout: Meta<Ref<super::Definition<M>>, M>,
	doc: Documentation,
}

/// Layout field parts.
#[derive(Clone)]
pub struct FieldsParts<M> {
	pub prop: MetaOption<Ref<prop::Definition<M>>, M>,
	pub name: Meta<Name, M>,
	pub label: Option<String>,
	pub layout: Meta<Ref<super::Definition<M>>, M>,
	pub doc: Documentation,
}

impl<M> Field<M> {
	pub fn new(
		prop: MetaOption<Ref<prop::Definition<M>>, M>,
		name: Meta<Name, M>,
		label: Option<String>,
		layout: Meta<Ref<super::Definition<M>>, M>,
		doc: Documentation,
	) -> Self {
		Self {
			prop,
			name,
			label,
			layout,
			doc,
		}
	}

	pub fn into_parts(self) -> FieldsParts<M> {
		FieldsParts {
			prop: self.prop,
			name: self.name,
			label: self.label,
			layout: self.layout,
			doc: self.doc,
		}
	}

	pub fn property(&self) -> Option<Ref<prop::Definition<M>>> {
		self.prop.value().cloned()
	}

	pub fn property_with_causes(&self) -> &MetaOption<Ref<prop::Definition<M>>, M> {
		&self.prop
	}

	pub fn name(&self) -> &Name {
		&self.name
	}

	pub fn name_with_causes(&self) -> &Meta<Name, M> {
		&self.name
	}

	pub fn label(&self) -> Option<&str> {
		self.label.as_deref()
	}

	pub fn preferred_label<'a>(&'a self, model: &'a crate::Model<M>) -> Option<&'a str> {
		if self.label.is_none() {
			self.property().and_then(|prop_ref| {
				let prop_id = model.properties().get(prop_ref).unwrap().id();
				model.get(prop_id).unwrap().label()
			})
		} else {
			self.label.as_deref()
		}
	}

	pub fn layout(&self) -> Ref<layout::Definition<M>> {
		*self.layout
	}

	pub fn layout_with_causes(&self) -> &Meta<Ref<super::Definition<M>>, M> {
		&self.layout
	}

	pub fn is_required(&self, model: &crate::Model<M>) -> bool {
		let layout = model.layouts().get(self.layout()).unwrap();
		layout.is_required()
	}

	pub fn set_layout(&mut self, layout: Ref<layout::Definition<M>>, metadata: M) {
		self.layout = Meta::new(layout, metadata)
	}

	pub fn documentation(&self) -> &Documentation {
		&self.doc
	}

	pub fn preferred_documentation<'a>(&'a self, model: &'a crate::Model<M>) -> &'a Documentation {
		if self.doc.is_empty() && self.prop.is_some() {
			let prop_id = model
				.properties()
				.get(*self.prop.value().unwrap())
				.unwrap()
				.id();
			model.get(prop_id).unwrap().documentation()
		} else {
			&self.doc
		}
	}
}
