use crate::{
	error, layout, prop, vocab::Name, Caused, Documentation, Error, Id, MaybeSet, WithCauses,
};
use locspan::Location;
use shelves::Ref;

/// Structure layout.
#[derive(Clone)]
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

	pub fn intersected_with(
		self,
		id: Id,
		other: &Self,
		name: MaybeSet<Name, F>,
		cause: Option<&Location<F>>,
	) -> Result<Self, Error<F>>
	where
		F: Clone + Ord,
	{
		let mut fields = Vec::new();

		let mut j = 0;
		for field in &self.fields {
			for (k, other_field) in other.fields[j..].iter().enumerate() {
				if field.name() == other_field.name() {
					if field.property() != other_field.property() {
						return Err(Caused::new(
							error::LayoutIntersectionFailed { id }.into(),
							cause.cloned(),
						));
					}

					if field.layout() != other_field.layout() {
						return Err(Caused::new(
							error::LayoutIntersectionFailed { id }.into(),
							cause.cloned(),
						));
					}

					let required = if *field.required || !*other_field.required {
						field.required.clone()
					} else {
						other_field.required.clone()
					};

					let functional = if !*field.functional && *other_field.functional {
						field.functional.clone()
					} else {
						other_field.functional.clone()
					};

					let doc = if field.doc.is_empty() || other_field.doc.is_empty() {
						field.doc.clone()
					} else {
						other_field.doc.clone()
					};

					fields.push(Field {
						prop: field.prop.clone(),
						name: field.name.clone(),
						label: field.label.clone().or_else(|| other_field.label.clone()),
						layout: field.layout.clone(),
						required,
						functional,
						doc,
					});

					j += k;
				}
			}
		}

		Ok(Self {
			name: name.unwrap().unwrap_or(self.name),
			fields,
		})
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
