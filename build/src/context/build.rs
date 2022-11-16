use locspan::Meta;
use rdf_types::{VocabularyMut, Generator};
use shelves::Shelf;
use treeldr::{IriIndex, BlankIdIndex, Model, metadata::Merge};

use crate::{Context, Error};

mod unify;
mod compute_layouts_relations;
mod assign_default_layouts;
mod assign_default_names;

pub use compute_layouts_relations::LayoutRelations;

impl<M: Clone> Context<M> {
	pub fn build<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		mut self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<Model<M>, Error<M>>
	where
		M: Clone + Merge,
	{
		use crate::utils::SccGraph;
		use crate::Build;

		self.unify();
		self.assign_default_layouts(vocabulary, generator);
		let layouts_relations = self.compute_layouts_relations();
		self.assign_default_names(vocabulary, &layouts_relations);

		// let mut allocated_shelves = allocated::Shelves::default();
		// let mut allocated_nodes = allocated::Nodes::new(&mut allocated_shelves, self.nodes);
		// let graph = allocated_shelves.dependency_graph(&allocated_nodes)?;

		// let components = graph.strongly_connected_components();

		// let ordered_components = components.order_by_depth();

		// let mut types_to_build: Vec<_> = allocated_shelves
		// 	.types
		// 	.into_storage()
		// 	.into_iter()
		// 	.map(Option::Some)
		// 	.collect();
		// let mut properties_to_build: Vec<_> = allocated_shelves
		// 	.properties
		// 	.into_storage()
		// 	.into_iter()
		// 	.map(Option::Some)
		// 	.collect();
		// let mut layouts_to_build: Vec<_> = allocated_shelves
		// 	.layouts
		// 	.into_storage()
		// 	.into_iter()
		// 	.map(Option::Some)
		// 	.collect();

		// let mut built_types = Vec::new();
		// built_types.resize_with(types_to_build.len(), || None);
		// let mut built_properties = Vec::new();
		// built_properties.resize_with(properties_to_build.len(), || None);
		// let mut built_layouts = Vec::new();
		// built_layouts.resize_with(layouts_to_build.len(), || None);

		// for i in ordered_components.into_iter().rev() {
		// 	let component = components.get(i).unwrap();
		// 	for item in component {
		// 		let dependencies = crate::Dependencies {
		// 			types: &built_types,
		// 			properties: &built_properties,
		// 			layouts: &built_layouts,
		// 		};

		// 		match item {
		// 			crate::Item::Type(ty_ref) => {
		// 				let (_, Meta(ty, causes)) = types_to_build[ty_ref.index()].take().unwrap();
		// 				let built_ty = ty.build(&mut allocated_nodes, dependencies, causes)?;
		// 				built_types[ty_ref.index()] = Some(built_ty)
		// 			}
		// 			crate::Item::Property(prop_ref) => {
		// 				let (_, Meta(prop, causes)) =
		// 					properties_to_build[prop_ref.index()].take().unwrap();
		// 				let built_prop = prop.build(&mut allocated_nodes, dependencies, causes)?;
		// 				built_properties[prop_ref.index()] = Some(built_prop)
		// 			}
		// 			crate::Item::Layout(layout_ref) => {
		// 				let (_, Meta(layout, causes)) =
		// 					layouts_to_build[layout_ref.index()].take().unwrap();
		// 				let built_layout =
		// 					layout.build(&mut allocated_nodes, dependencies, causes)?;
		// 				built_layouts[layout_ref.index()] = Some(built_layout)
		// 			}
		// 		}
		// 	}
		// }

		// let mut model = Model::from_parts(
		// 	allocated_nodes.into_model_nodes(),
		// 	Shelf::new(built_types.into_iter().map(Option::unwrap).collect()),
		// 	Shelf::new(built_properties.into_iter().map(Option::unwrap).collect()),
		// 	Shelf::new(built_layouts.into_iter().map(Option::unwrap).collect()),
		// );

		// model.simplify(vocabulary, generator);

		// Ok(model)
		todo!()
	}
}
