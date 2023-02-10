use std::collections::BTreeMap;

use rdf_types::{Generator, VocabularyMut};
use treeldr::{metadata::Merge, BlankIdIndex, IriIndex};

use crate::Context;

impl<M> Context<M> {
	pub fn assign_default_layouts<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) where
		M: Clone + Merge,
	{
		let mut default_layouts = BTreeMap::new();
		for (id, node) in &self.nodes {
			if node.as_formatted().format().is_empty() {
				if let Some(default_layout) = node.as_layout_field().default_layout(self) {
					default_layouts.insert(*id, default_layout);
				}
			}
		}

		for (id, default_layout) in default_layouts {
			let default_layout = default_layout.build(self, vocabulary, generator);
			self.get_mut(id)
				.unwrap()
				.as_formatted_mut()
				.format_mut()
				.insert_base(default_layout)
		}
	}
}
