use std::collections::HashMap;

use crate::{MetaOption, Name, Layout, TId};
use locspan::Meta;

/// Enum layout.
#[derive(Debug, Clone)]
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

	pub fn can_be_reference(
		&self,
		map: &mut HashMap<TId<Layout>, bool>,
		model: &crate::Model<M>,
	) -> bool {
		for v in &self.variants {
			if let Some(r) = v.layout() {
				if model.can_be_reference_layout(map, r) {
					return true;
				}
			}
		}

		false
	}
}

#[derive(Debug, Clone)]
pub struct Variant<M> {
	name: Meta<Name, M>,
	layout: MetaOption<TId<Layout>, M>
}

pub struct VariantParts<M> {
	pub name: Meta<Name, M>,
	pub layout: MetaOption<TId<Layout>, M>
}

impl<M> Variant<M> {
	pub fn new(
		name: Meta<Name, M>,
		layout: MetaOption<TId<Layout>, M>
	) -> Self {
		Self {
			name,
			layout
		}
	}

	pub fn into_parts(self) -> VariantParts<M> {
		VariantParts {
			name: self.name,
			layout: self.layout
		}
	}

	pub fn name(&self) -> &Name {
		&self.name
	}

	pub fn layout(&self) -> Option<TId<Layout>> {
		self.layout.value().cloned()
	}
}

pub struct ComposingLayouts<'a, M>(std::slice::Iter<'a, Meta<Variant<M>, M>>);

impl<'a, M> Iterator for ComposingLayouts<'a, M> {
	type Item = TId<Layout>;

	fn next(&mut self) -> Option<Self::Item> {
		for variant in self.0.by_ref() {
			if let Some(layout_ref) = variant.layout() {
				return Some(layout_ref);
			}
		}

		None
	}
}
