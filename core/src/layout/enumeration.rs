use crate::{Documentation, MetaOption, Name, Ref};
use locspan::Meta;

/// Enum layout.
#[derive(Clone)]
pub struct Enum<M> {
	name: Meta<Name, M>,
	variants: Vec<Meta<Variant<M>, M>>,
}

pub struct Parts<M> {
	pub name: Meta<Name, M>,
	pub variants: Vec<Meta<Variant<M>, M>>,
}

impl<M> Enum<M> {
	pub fn new(name: Meta<Name, M>, variants: Vec<Meta<Variant<M>, M>>) -> Self {
		Self { name, variants }
	}

	pub fn into_parts(self) -> Parts<M> {
		Parts {
			name: self.name,
			variants: self.variants,
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

	pub fn variants(&self) -> &[Meta<Variant<M>, M>] {
		&self.variants
	}

	pub fn composing_layouts(&self) -> ComposingLayouts<M> {
		ComposingLayouts(self.variants.iter())
	}
}

#[derive(Clone)]
pub struct Variant<M> {
	name: Meta<Name, M>,
	layout: MetaOption<Ref<super::Definition<M>>, M>,
	label: Option<String>,
	doc: Documentation,
}

pub struct VariantParts<M> {
	pub name: Meta<Name, M>,
	pub layout: MetaOption<Ref<super::Definition<M>>, M>,
	pub label: Option<String>,
	pub doc: Documentation,
}

impl<M> Variant<M> {
	pub fn new(
		name: Meta<Name, M>,
		layout: MetaOption<Ref<super::Definition<M>>, M>,
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

	pub fn into_parts(self) -> VariantParts<M> {
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

	pub fn layout(&self) -> Option<Ref<super::Definition<M>>> {
		self.layout.value().cloned()
	}

	pub fn documentation(&self) -> &Documentation {
		&self.doc
	}
}

pub struct ComposingLayouts<'a, M>(std::slice::Iter<'a, Meta<Variant<M>, M>>);

impl<'a, M> Iterator for ComposingLayouts<'a, M> {
	type Item = Ref<super::Definition<M>>;

	fn next(&mut self) -> Option<Self::Item> {
		for variant in self.0.by_ref() {
			if let Some(layout_ref) = variant.layout() {
				return Some(layout_ref);
			}
		}

		None
	}
}

// pub struct Fields<'a, M> {
// 	variants: std::slice::Iter<'a, Meta<Ref<super::Definition<M>>, M>>,
// 	current_fields: Option<std::slice::Iter<'a, Field<M>>>,
// }

// impl<'a, M> Iterator for Fields<'a, M> {
// 	type Item = &'a Field<M>;

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
