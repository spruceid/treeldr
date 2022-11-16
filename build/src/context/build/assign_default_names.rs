use std::collections::{BTreeMap, HashMap};

use rdf_types::Vocabulary;
use treeldr::{IriIndex, BlankIdIndex, Id, metadata::Merge};

use crate::Context;
use super::LayoutRelations;

impl<M: Clone + Merge> Context<M> {
	/// Assigns default name for layouts/variants that don't have a name yet.
	pub fn assign_default_names(
		&mut self,
		vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
		layout_relations: &HashMap<Id, LayoutRelations<M>>
	) {
		// Start with the fields.
		let mut default_field_names = BTreeMap::new();
		for (id, node) in &self.nodes {
			if let Some(field) = node.as_layout_field() {
				if let Some(name) = field.default_name(vocabulary, field.metadata().clone()) {
					default_field_names.insert(*id, name);
				}
			}
		}
		for (id, name) in default_field_names {
			let field = self.get_mut(id).unwrap().as_layout_field_mut().unwrap();
			if field.name().is_empty() {
				field.name_mut().insert(name);
			}
		}

		// Now the layouts.
		use treeldr::utils::SccGraph;
		struct LayoutGraph {
			layouts: Vec<Id>,
			dependencies: HashMap<Id, Vec<Id>>,
		}

		impl SccGraph for LayoutGraph {
			type Vertex = Id;

			fn vertices(&self) -> &[Self::Vertex] {
				&self.layouts
			}

			fn successors(&self, v: Self::Vertex) -> &[Self::Vertex] {
				self.dependencies.get(&v).unwrap()
			}
		}

		// Compute layout parent-child graph.
		let mut graph = LayoutGraph {
			layouts: Vec::new(),
			dependencies: HashMap::new(),
		};

		for (id, node) in &self.nodes {
			if node.is_layout() {
				let parent_layouts = &layout_relations.get(id).unwrap().parent;
				let dependencies: Vec<_> = parent_layouts.iter().map(|p| p.layout).collect();
				graph.layouts.push(*id);
				graph.dependencies.insert(*id, dependencies);
			}
		}

		let components = graph.strongly_connected_components();
		let ordered_components = components.order_by_depth();
		for i in ordered_components.into_iter().rev() {
			let component = components.get(i).unwrap();
			for id in component {
				let layout = self.nodes.get(id).unwrap().as_layout().unwrap();
				let parent_layouts = &layout_relations.get(id).unwrap().parent;
				if let Some(name) = layout.default_name(
					self,
					vocabulary,
					parent_layouts,
					layout.metadata().clone(),
				) {
					let layout = self.get_mut(*id).unwrap().as_layout_mut().unwrap();
					if layout.name().is_empty() {
						layout.name_mut().insert(name);
					}
				}
			}
		}

		// Now the layouts variants.
		let mut default_variant_names = BTreeMap::new();
		for (id, node) in &self.nodes {
			if let Some(layout) = node.as_layout_variant() {
				if let Some(name) =
					layout.default_name(self, vocabulary, layout.metadata().clone())
				{
					default_variant_names.insert(*id, name);
				}
			}
		}
		for (id, name) in default_variant_names {
			let variant = self.get_mut(id).unwrap().as_layout_variant_mut().unwrap();
			if variant.name().is_empty() {
				variant.name_mut().insert(name);
			}
		}
	}
}