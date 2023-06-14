use std::collections::BTreeMap;

use locspan::MapLocErr;
use rdf_types::Generator;
use treeldr::{metadata::Merge, vocab::TldrVocabulary, Model, MutableModel};

use crate::{error, ty::ClassHierarchy, Context, Error};

mod assign_default_layouts;
mod assign_default_names;
mod close_layout_types;
mod compute_layout_intersections;
mod compute_layouts_relations;
mod dispatch_sub_properties;
mod minimize;
mod remove_unused_nodes;
mod simplify_composite_types_and_layouts;
mod unify;

pub use compute_layouts_relations::LayoutRelations;

impl<M: Clone> Context<M> {
	pub fn build(
		&mut self,
		vocabulary: &mut TldrVocabulary,
		generator: &mut impl Generator<TldrVocabulary>,
	) -> Result<Model<M>, Error<M>>
	where
		M: Clone + Merge,
	{
		// TODO check for infinite values (lists).

		log::debug!("close layout `tldr:layoutFor` relation by aliasing...");
		self.close_layout_types();

		log::debug!("dispatch sub properties...");
		self.dispatch_sub_properties()?;

		log::debug!("computing layout intersections...");
		self.compute_layout_intersections(vocabulary, generator)?;

		log::debug!("unifying blank nodes...");
		self.unify(vocabulary, generator);

		log::debug!("simplifying composite types and layouts...");
		self.simplify_composite_types_and_layouts();
		self.remove_unused_nodes();

		log::debug!("class hierarchy analysis...");
		let class_hierarchy = ClassHierarchy::new(self).map_loc_err(error::Description::from)?;
		class_hierarchy.apply(self);

		log::debug!("minimizing `rdfs:domain` and `rdfs:range`...");
		self.minimize_domain_and_range();

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

		let mutable_model = MutableModel::from_parts(nodes);

		log::debug!("post build analysis...");
		let model = Model::new(mutable_model).expect("property restriction contradiction");

		log::debug!("done.");
		Ok(model)
	}
}
