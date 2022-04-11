use crate::{layout, prop, vocab::Name, Documentation, MaybeSet, WithCauses};
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

	pub fn as_sum_option(&self) -> Option<Ref<super::Definition<F>>> {
		if self.fields.len() == 1 {
			Some(self.fields[0].layout())
		} else {
			None
		}
	}
}

/// Layout field.
pub struct Field<F> {
	prop: MaybeSet<Ref<prop::Definition<F>>, F>,
	name: WithCauses<Name, F>,
	label: Option<String>,
	layout: WithCauses<Ref<super::Definition<F>>, F>,
	required: WithCauses<bool, F>,
	// functional: WithCauses<bool, F>,
	doc: Documentation,
}

impl<F> Field<F> {
	pub fn new(
		prop: MaybeSet<Ref<prop::Definition<F>>, F>,
		name: WithCauses<Name, F>,
		label: Option<String>,
		layout: WithCauses<Ref<super::Definition<F>>, F>,
		required: WithCauses<bool, F>,
		// functional: WithCauses<bool, F>,
		doc: Documentation,
	) -> Self {
		Self {
			prop,
			name,
			label,
			layout,
			required,
			// functional,
			doc,
		}
	}

	pub fn property(&self) -> Option<Ref<prop::Definition<F>>> {
		self.prop.value().cloned()
	}

	pub fn name(&self) -> &Name {
		&self.name
	}

	pub fn label(&self) -> Option<&str> {
		self.label.as_deref()
	}

	pub fn preferred_label<'a>(&'a self, model: &'a crate::Model<F>) -> Option<&'a str> {
		if self.label.is_none() {
			self.property().and_then(|prop| {
				let prop_id = model.properties().get(prop).unwrap().id();
				model.get(prop_id).unwrap().label()
			})
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

	// pub fn is_functional(&self) -> bool {
	// 	*self.functional.inner()
	// }

	pub fn documentation(&self) -> &Documentation {
		&self.doc
	}

	pub fn preferred_documentation<'a>(&'a self, model: &'a crate::Model<F>) -> &'a Documentation {
		if self.doc.is_empty() {
			match self.property() {
				Some(prop) => {
					let prop_id = model.properties().get(prop).unwrap().id();
					model.get(prop_id).unwrap().documentation()
				}
				None => &self.doc,
			}
		} else {
			&self.doc
		}
	}
}
