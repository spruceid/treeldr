use std::collections::BTreeMap;

use rdf_types::{Generator, VocabularyMut};
use treeldr::{metadata::Merge, BlankIdIndex, IriIndex, Model};

use crate::{Context, Error};

mod assign_default_layouts;
mod assign_default_names;
mod compute_layout_intersections;
mod compute_layouts_relations;
mod remove_unused_nodes;
mod simplify_composite_types_and_layouts;
mod unify;

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
		// TODO check for infinite values (lists).

		log::debug!("computing layout intersections...");
		self.compute_layout_intersections(vocabulary, generator)?;

		log::debug!("simplifying composite types and layouts...");
		self.simplify_composite_types_and_layouts();
		self.remove_unused_nodes();

		log::debug!("unifying blank nodes...");
		self.unify(vocabulary, generator);

		log::debug!("simplifying composite types and layouts...");
		self.simplify_composite_types_and_layouts();
		self.remove_unused_nodes();

		log::debug!("assigning default layouts...");
		self.assign_default_layouts(vocabulary, generator);

		log::debug!("computing layouts relations...");
		let layouts_relations = self.compute_layouts_relations();

		log::debug!("assigning default component names...");
		self.assign_default_names(vocabulary, &layouts_relations);

		log::debug!("building...");
		let mut nodes = BTreeMap::new();
		for (id, node) in &self.nodes {
			if let Some(node) = node.build(self)? {
				nodes.insert(*id, node);
			}
		}

		log::debug!("done.");
		Ok(Model::from_parts(nodes))
	}
}
