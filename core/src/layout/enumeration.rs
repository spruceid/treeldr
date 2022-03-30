use super::Field;
use crate::{Documentation, MaybeSet, WithCauses};

/// Enum layout.
pub struct Enum<F> {
	name: WithCauses<String, F>,
	variants: Vec<Variant<F>>,
}

impl<F> Enum<F> {
	pub fn name(&self) -> &str {
		&self.name
	}

	pub fn variants(&self) -> &[Variant<F>] {
		&self.variants
	}

	pub fn fields(&self) -> Fields<F> {
		Fields {
			variants: self.variants.iter(),
			current_fields: None,
		}
	}
}

/// Enum layout variant.
pub struct Variant<F> {
	name: WithCauses<String, F>,
	label: Option<String>,
	payload: MaybeSet<Vec<Field<F>>, F>,
	doc: Documentation,
}

impl<F> Variant<F> {
	pub fn new(
		name: WithCauses<String, F>,
		label: Option<String>,
		payload: MaybeSet<Vec<Field<F>>, F>,
		doc: Documentation,
	) -> Self {
		Self {
			name,
			label,
			payload,
			doc,
		}
	}

	pub fn name(&self) -> &str {
		&self.name
	}

	pub fn label(&self) -> Option<&str> {
		self.label.as_deref()
	}

	pub fn payload(&self) -> Option<&[Field<F>]> {
		self.payload.value().map(Vec::as_slice)
	}

	pub fn documentation(&self) -> &Documentation {
		&self.doc
	}
}

pub struct Fields<'a, F> {
	variants: std::slice::Iter<'a, Variant<F>>,
	current_fields: Option<std::slice::Iter<'a, Field<F>>>,
}

impl<'a, F> Iterator for Fields<'a, F> {
	type Item = &'a Field<F>;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			match self.current_fields.as_mut().map(Iterator::next) {
				Some(Some(item)) => break Some(item),
				Some(None) => self.current_fields = None,
				None => match self.variants.next() {
					Some(variant) => self.current_fields = variant.payload().map(|f| f.iter()),
					None => break None,
				},
			}
		}
	}
}
