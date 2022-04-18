use crate::{
	error, vocab::Name, Caused, Causes, Documentation, Error, Id, MaybeSet, Ref, WithCauses,
};
use locspan::Location;

/// Enum layout.
#[derive(Clone)]
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
		let mut variants = Vec::new();

		let mut j = 0;
		for variant in &self.variants {
			for (k, other_variant) in other.variants[j..].iter().enumerate() {
				if variant.name() == other_variant.name() {
					if variant.layout() != other_variant.layout() {
						return Err(Caused::new(
							error::LayoutIntersectionFailed { id }.into(),
							cause.cloned(),
						));
					}

					let doc = if variant.doc.is_empty() || other_variant.doc.is_empty() {
						variant.doc.clone()
					} else {
						other_variant.doc.clone()
					};

					variants.push(WithCauses::new(
						Variant {
							name: variant.name.clone(),
							label: variant
								.label
								.clone()
								.or_else(|| other_variant.label.clone()),
							layout: variant.layout.clone(),
							doc,
						},
						variant
							.causes()
							.clone()
							.with(other_variant.causes().iter().cloned()),
					));

					j += k;
				}
			}
		}

		Ok(Self {
			name: name.unwrap().unwrap_or(self.name),
			variants,
		})
	}
}

#[derive(Clone)]
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
