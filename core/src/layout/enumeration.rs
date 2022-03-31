use crate::{vocab::Name, Documentation, MaybeSet, Ref, WithCauses};

/// Enum layout.
pub struct Enum<F> {
	name: WithCauses<Name, F>,
	variants: Vec<WithCauses<Variant<F>, F>>,
}

impl<F> Enum<F> {
	pub fn new(name: WithCauses<Name, F>, variants: Vec<WithCauses<Variant<F>, F>>) -> Self {
		Self { name, variants }
	}

	pub fn name(&self) -> &Name {
		&self.name
	}

	pub fn variants(&self) -> &[WithCauses<Variant<F>, F>] {
		&self.variants
	}

	// pub fn fields(&self) -> Fields<F> {
	// 	Fields {
	// 		variants: self.variants.iter(),
	// 		current_fields: None,
	// 	}
	// }

	pub fn composing_layouts(&self) -> ComposingLayouts<F> {
		ComposingLayouts(self.variants.iter())
	}
}

pub struct Variant<F> {
	name: WithCauses<Name, F>,
	layout: MaybeSet<Ref<super::Definition<F>>, F>,
	label: Option<String>,
	doc: Documentation,
}

impl<F> Variant<F> {
	pub fn new(
		name: WithCauses<Name, F>,
		layout: MaybeSet<Ref<super::Definition<F>>, F>,
		label: Option<String>,
		doc: Documentation,
	) -> Self {
		Self {
			name,
			layout,
			label,
			doc,
		}
	}

	pub fn name(&self) -> &Name {
		&self.name
	}

	pub fn label(&self) -> Option<&str> {
		self.label.as_deref()
	}

	pub fn layout(&self) -> Option<Ref<super::Definition<F>>> {
		self.layout.value().cloned()
	}

	pub fn documentation(&self) -> &Documentation {
		&self.doc
	}
}

pub struct ComposingLayouts<'a, F>(std::slice::Iter<'a, WithCauses<Variant<F>, F>>);

impl<'a, F> Iterator for ComposingLayouts<'a, F> {
	type Item = Ref<super::Definition<F>>;

	fn next(&mut self) -> Option<Self::Item> {
		for variant in self.0.by_ref() {
			if let Some(layout_ref) = variant.layout() {
				return Some(layout_ref);
			}
		}

		None
	}
}

// pub struct Fields<'a, F> {
// 	variants: std::slice::Iter<'a, WithCauses<Ref<super::Definition<F>>, F>>,
// 	current_fields: Option<std::slice::Iter<'a, Field<F>>>,
// }

// impl<'a, F> Iterator for Fields<'a, F> {
// 	type Item = &'a Field<F>;

// 	fn next(&mut self) -> Option<Self::Item> {
// 		loop {
// 			match self.current_fields.as_mut().map(Iterator::next) {
// 				Some(Some(item)) => break Some(item),
// 				Some(None) => self.current_fields = None,
// 				None => match self.variants.next() {
// 					Some(variant) => self.current_fields = Some(variant.fields().iter()),
// 					None => break None,
// 				},
// 			}
// 		}
// 	}
// }
