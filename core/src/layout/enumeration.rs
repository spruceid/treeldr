use crate::{Causes, Documentation, MaybeSet, Name, Ref, WithCauses};
use locspan::Location;

/// Enum layout.
#[derive(Clone)]
pub struct Enum<F> {
	name: WithCauses<Name, F>,
	variants: Vec<WithCauses<Variant<F>, F>>,
}

pub struct Parts<F> {
	pub name: WithCauses<Name, F>,
	pub variants: Vec<WithCauses<Variant<F>, F>>,
}

impl<F> Enum<F> {
	pub fn new(name: WithCauses<Name, F>, variants: Vec<WithCauses<Variant<F>, F>>) -> Self {
		Self { name, variants }
	}

	pub fn into_parts(self) -> Parts<F> {
		Parts {
			name: self.name,
			variants: self.variants,
		}
	}

	pub fn name(&self) -> &Name {
		&self.name
	}

	pub fn into_name(self) -> WithCauses<Name, F> {
		self.name
	}

	pub fn name_causes(&self) -> &Causes<F> {
		self.name.causes()
	}

	pub fn set_name(&mut self, new_name: Name, cause: Option<Location<F>>) -> WithCauses<Name, F>
	where
		F: Ord,
	{
		std::mem::replace(&mut self.name, WithCauses::new(new_name, cause))
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

#[derive(Clone)]
pub struct Variant<F> {
	name: WithCauses<Name, F>,
	layout: MaybeSet<Ref<super::Definition<F>>, F>,
	label: Option<String>,
	doc: Documentation,
}

pub struct VariantParts<F> {
	pub name: WithCauses<Name, F>,
	pub layout: MaybeSet<Ref<super::Definition<F>>, F>,
	pub label: Option<String>,
	pub doc: Documentation,
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

	pub fn into_parts(self) -> VariantParts<F> {
		VariantParts {
			name: self.name,
			layout: self.layout,
			label: self.label,
			doc: self.doc,
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
