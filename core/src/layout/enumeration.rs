use std::collections::HashMap;

use crate::{Layout, TId};
use locspan::Meta;

use super::Variant;

/// Enum layout.
#[derive(Debug, Clone)]
pub struct Enum<M> {
	variants: Vec<Meta<TId<Variant>, M>>,
}

pub struct Parts<M> {
	pub variants: Vec<Meta<TId<Variant>, M>>,
}

impl<M> Enum<M> {
	pub fn new(variants: Vec<Meta<TId<Variant>, M>>) -> Self {
		Self { variants }
	}

	pub fn variants(&self) -> &[Meta<TId<Variant>, M>] {
		&self.variants
	}

	pub fn composing_layouts<'a>(&'a self, model: &'a crate::Model<M>) -> ComposingLayouts<'a, M> {
		ComposingLayouts(model, self.variants.iter())
	}

	pub fn can_be_reference(
		&self,
		map: &mut HashMap<TId<Layout>, bool>,
		model: &crate::Model<M>,
	) -> bool {
		for v in &self.variants {
			if let Some(r) = model.get(**v).unwrap().as_formatted().format().value() {
				if model.can_be_reference_layout(map, *r) {
					return true;
				}
			}
		}

		false
	}
}

pub struct ComposingLayouts<'a, M>(
	&'a crate::Model<M>,
	std::slice::Iter<'a, Meta<TId<Variant>, M>>,
);

impl<'a, M> Iterator for ComposingLayouts<'a, M> {
	type Item = &'a Meta<TId<Layout>, M>;

	fn next(&mut self) -> Option<Self::Item> {
		for variant in self.1.by_ref() {
			if let Some(layout_ref) = self
				.0
				.get(**variant)
				.unwrap()
				.as_formatted()
				.format()
				.as_ref()
			{
				return Some(layout_ref);
			}
		}

		None
	}
}
