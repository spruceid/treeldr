use crate::{layout, prop, vocab::Name, Documentation, WithCauses};
use locspan::Location;
use shelves::Ref;

/// Structure layout.
#[derive(Clone)]
pub struct Struct<F> {
	name: WithCauses<Name, F>,
	fields: Vec<Field<F>>,
}

/// Structure layout parts.
#[derive(Clone)]
pub struct Parts<F> {
	pub name: WithCauses<Name, F>,
	pub fields: Vec<Field<F>>,
}

impl<F> Struct<F> {
	pub fn new(name: WithCauses<Name, F>, fields: Vec<Field<F>>) -> Self {
		Self { name, fields }
	}

	pub fn into_parts(self) -> Parts<F> {
		unsafe { std::mem::transmute(self) }
	}

	pub fn name(&self) -> &Name {
		&self.name
	}

	pub fn set_name(&mut self, new_name: Name, cause: Option<Location<F>>) -> WithCauses<Name, F>
	where
		F: Ord,
	{
		std::mem::replace(&mut self.name, WithCauses::new(new_name, cause))
	}

	pub fn fields(&self) -> &[Field<F>] {
		&self.fields
	}

	pub fn as_sum_option(&self) -> Option<Ref<super::Definition<F>>> {
		if self.fields.len() == 1 {
			Some(self.fields[0].layout())
		} else {
			None
		}
	}
}

/// Layout field.
#[derive(Clone)]
pub struct Field<F> {
	prop: WithCauses<Ref<prop::Definition<F>>, F>,
	name: WithCauses<Name, F>,
	label: Option<String>,
	layout: WithCauses<Ref<super::Definition<F>>, F>,
	required: WithCauses<bool, F>,
	functional: WithCauses<bool, F>,
	doc: Documentation,
}

/// Layout field parts.
#[derive(Clone)]
pub struct FieldsParts<F> {
	pub prop: WithCauses<Ref<prop::Definition<F>>, F>,
	pub name: WithCauses<Name, F>,
	pub label: Option<String>,
	pub layout: WithCauses<Ref<super::Definition<F>>, F>,
	pub required: WithCauses<bool, F>,
	pub functional: WithCauses<bool, F>,
	pub doc: Documentation,
}

impl<F> Field<F> {
	pub fn new(
		prop: WithCauses<Ref<prop::Definition<F>>, F>,
		name: WithCauses<Name, F>,
		label: Option<String>,
		layout: WithCauses<Ref<super::Definition<F>>, F>,
		required: WithCauses<bool, F>,
		functional: WithCauses<bool, F>,
		doc: Documentation,
	) -> Self {
		Self {
			prop,
			name,
			label,
			layout,
			required,
			functional,
			doc,
		}
	}

	pub fn into_parts(self) -> FieldsParts<F> {
		unsafe { std::mem::transmute(self) }
	}

	pub fn property(&self) -> Ref<prop::Definition<F>> {
		*self.prop.inner()
	}

	pub fn property_with_causes(&self) -> &WithCauses<Ref<prop::Definition<F>>, F> {
		&self.prop
	}

	pub fn name(&self) -> &Name {
		&self.name
	}

	pub fn name_with_causes(&self) -> &WithCauses<Name, F> {
		&self.name
	}

	pub fn label(&self) -> Option<&str> {
		self.label.as_deref()
	}

	pub fn preferred_label<'a>(&'a self, model: &'a crate::Model<F>) -> Option<&'a str> {
		if self.label.is_none() {
			let prop_id = model.properties().get(*self.prop).unwrap().id();
			model.get(prop_id).unwrap().label()
		} else {
			self.label.as_deref()
		}
	}

	pub fn layout(&self) -> Ref<layout::Definition<F>> {
		*self.layout.inner()
	}

	pub fn layout_with_causes(&self) -> &WithCauses<Ref<super::Definition<F>>, F> {
		&self.layout
	}

	pub fn is_required(&self) -> bool {
		*self.required.inner()
	}

	pub fn is_required_with_causes(&self) -> &WithCauses<bool, F> {
		&self.required
	}

	pub fn is_functional(&self) -> bool {
		*self.functional.inner()
	}

	pub fn is_functional_with_causes(&self) -> &WithCauses<bool, F> {
		&self.functional
	}

	pub fn documentation(&self) -> &Documentation {
		&self.doc
	}

	pub fn preferred_documentation<'a>(&'a self, model: &'a crate::Model<F>) -> &'a Documentation {
		if self.doc.is_empty() {
			let prop_id = model.properties().get(*self.prop).unwrap().id();
			model.get(prop_id).unwrap().documentation()
		} else {
			&self.doc
		}
	}
}
