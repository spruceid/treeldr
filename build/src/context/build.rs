use std::collections::BTreeMap;

use rdf_types::{VocabularyMut, Generator};
use treeldr::{IriIndex, BlankIdIndex, Model, metadata::Merge};

use crate::{Context, Error};

mod unify;
mod compute_layouts_relations;
mod assign_default_layouts;
mod assign_default_names;

pub use compute_layouts_relations::LayoutRelations;

impl<M: Clone> Context<M> {
	pub fn build<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<Model<M>, Error<M>>
	where
		M: Clone + Merge,
	{
		self.unify();
		self.assign_default_layouts(vocabulary, generator);
		let layouts_relations = self.compute_layouts_relations();
		self.assign_default_names(vocabulary, &layouts_relations);

		let mut nodes = BTreeMap::new();
		for (id, node) in &self.nodes {
			nodes.insert(*id, node.build(self)?);
		}

		Ok(Model::from_parts(nodes))
	}
}