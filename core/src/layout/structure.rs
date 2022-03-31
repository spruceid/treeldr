use crate::{layout, prop, vocab::Name, Documentation, WithCauses};
use shelves::Ref;

/// Structure layout.
pub struct Struct<F> {
	name: WithCauses<Name, F>,
	fields: Vec<Field<F>>,
}

impl<F> Struct<F> {
	pub fn new(name: WithCauses<Name, F>, fields: Vec<Field<F>>) -> Self {
		Self { name, fields }
	}

	pub fn name(&self) -> &Name {
		&self.name
	}

	pub fn fields(&self) -> &[Field<F>] {
		&self.fields
	}
}

/// Layout field.
pub struct Field<F> {
	prop: WithCauses<Ref<prop::Definition<F>>, F>,
	name: WithCauses<Name, F>,
	label: Option<String>,
	layout: WithCauses<Ref<super::Definition<F>>, F>,
	required: WithCauses<bool, F>,
	functional: WithCauses<bool, F>,
	doc: Documentation,
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

	pub fn property(&self) -> Ref<prop::Definition<F>> {
		*self.prop.inner()
	}

	pub fn name(&self) -> &Name {
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

	pub fn is_required(&self) -> bool {
		*self.required.inner()
	}

	pub fn is_functional(&self) -> bool {
		*self.functional.inner()
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
