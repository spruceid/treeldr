use crate::{error, layout, list, node, prop, ty, utils::SccGraph, Error, ListRef, Node};
use derivative::Derivative;
use locspan::Meta;
use shelves::{Ref, Shelf};
use std::collections::BTreeMap;
use treeldr::{vocab, Id, IriIndex, MetaOption};

#[derive(Clone, Derivative)]
#[derivative(Default(bound = ""))]
pub struct Components<M> {
	ty: MetaOption<Ref<treeldr::ty::Definition<M>>, M>,
	property: MetaOption<Ref<treeldr::prop::Definition<M>>, M>,
	layout: MetaOption<Ref<treeldr::layout::Definition<M>>, M>,
	layout_field: MetaOption<layout::field::Definition<M>, M>,
	layout_variant: MetaOption<layout::variant::Definition<M>, M>,
	list: MetaOption<list::Definition<M>, M>,
}

impl<M> Node<Components<M>> {
	pub fn caused_types(&self) -> node::TypesMetadata<M>
	where
		M: Clone,
	{
		node::TypesMetadata {
			ty: self.value().ty.metadata().cloned(),
			property: self.value().property.metadata().cloned(),
			layout: self.value().property.metadata().cloned(),
			layout_field: self.value().layout_field.metadata().cloned(),
			layout_variant: self.value().layout_variant.metadata().cloned(),
			list: self.value().list.metadata().cloned(),
		}
	}

	pub fn require_type(
		&self,
		cause: &M,
	) -> Result<&Meta<Ref<treeldr::ty::Definition<M>>, M>, Error<M>>
	where
		M: Clone,
	{
		match self.value().ty.as_ref() {
			Some(ty) => Ok(ty),
			None => Err(Meta(
				error::NodeInvalidType {
					id: self.id(),
					expected: node::Type::Type,
					found: self.caused_types(),
				}
				.into(),
				cause.clone(),
			)),
		}
	}

	pub fn require_property(
		&self,
		cause: &M,
	) -> Result<&Meta<Ref<treeldr::prop::Definition<M>>, M>, Error<M>>
	where
		M: Clone,
	{
		match self.value().property.as_ref() {
			Some(prop) => Ok(prop),
			None => Err(Meta(
				error::NodeInvalidType {
					id: self.id(),
					expected: node::Type::Property,
					found: self.caused_types(),
				}
				.into(),
				cause.clone(),
			)),
		}
	}

	pub fn require_layout(
		&self,
		cause: &M,
	) -> Result<&Meta<Ref<treeldr::layout::Definition<M>>, M>, Error<M>>
	where
		M: Clone,
	{
		match self.value().layout.as_ref() {
			Some(layout) => Ok(layout),
			None => Err(Meta(
				error::NodeInvalidType {
					id: self.id(),
					expected: node::Type::Layout,
					found: self.caused_types(),
				}
				.into(),
				cause.clone(),
			)),
		}
	}

	pub fn require_layout_field(
		&self,
		cause: &M,
	) -> Result<&Meta<layout::field::Definition<M>, M>, Error<M>>
	where
		M: Clone,
	{
		match self.value().layout_field.as_ref() {
			Some(field) => Ok(field),
			None => Err(Meta(
				error::NodeInvalidType {
					id: self.id(),
					expected: node::Type::LayoutField,
					found: self.caused_types(),
				}
				.into(),
				cause.clone(),
			)),
		}
	}

	pub fn require_layout_variant(
		&self,
		cause: &M,
	) -> Result<&Meta<layout::variant::Definition<M>, M>, Error<M>>
	where
		M: Clone,
	{
		match self.value().layout_variant.as_ref() {
			Some(variant) => Ok(variant),
			None => Err(Meta(
				error::NodeInvalidType {
					id: self.id(),
					expected: node::Type::LayoutVariant,
					found: self.caused_types(),
				}
				.into(),
				cause.clone(),
			)),
		}
	}

	pub fn require_list(&self, cause: &M) -> Result<&Meta<list::Definition<M>, M>, Error<M>>
	where
		M: Clone,
	{
		match self.value().list.as_ref() {
			Some(list) => Ok(list),
			None => Err(Meta(
				error::NodeInvalidType {
					id: self.id(),
					expected: node::Type::List,
					found: self.caused_types(),
				}
				.into(),
				cause.clone(),
			)),
		}
	}
}

impl<M> From<Node<Components<M>>> for treeldr::Node<M> {
	fn from(n: Node<Components<M>>) -> treeldr::Node<M> {
		let (id, label, doc, value) = n.into_parts();

		treeldr::Node::from_parts(treeldr::node::Parts {
			id,
			label,
			ty: value.ty,
			property: value.property,
			layout: value.layout,
			doc,
		})
	}
}

#[allow(clippy::type_complexity)]
pub struct Shelves<M> {
	pub types: Shelf<Vec<(Id, Meta<ty::Definition<M>, M>)>>,
	pub properties: Shelf<Vec<(Id, Meta<prop::Definition<M>, M>)>>,
	pub layouts: Shelf<Vec<(Id, Meta<layout::Definition<M>, M>)>>,
}

impl<M> Default for Shelves<M> {
	fn default() -> Self {
		Self {
			types: Shelf::default(),
			properties: Shelf::default(),
			layouts: Shelf::default(),
		}
	}
}

impl<M: Clone> Shelves<M> {
	pub fn dependency_graph(
		&self,
		allocated_nodes: &Nodes<M>,
	) -> Result<DependencyGraph<M>, Error<M>> {
		let mut graph: DependencyGraph<M> = DependencyGraph {
			items: Vec::with_capacity(
				self.layouts.len() + self.properties.len() + self.types.len(),
			),
			ty_dependencies: Vec::with_capacity(self.types.len()),
			prop_dependencies: Vec::with_capacity(self.properties.len()),
			layout_dependencies: Vec::with_capacity(self.layouts.len()),
		};

		for (ty_ref, (_, ty)) in &self.types {
			graph.items.push(crate::Item::Type(ty_ref.cast()));
			let dependencies = ty.dependencies(allocated_nodes, ty.metadata())?;
			graph.ty_dependencies.push(dependencies)
		}

		for (prop_ref, (_, prop)) in &self.properties {
			graph.items.push(crate::Item::Property(prop_ref.cast()));
			let dependencies = prop.dependencies(allocated_nodes, prop.metadata())?;
			graph.prop_dependencies.push(dependencies)
		}

		for (layout_ref, (_, layout)) in &self.layouts {
			graph.items.push(crate::Item::Layout(layout_ref.cast()));
			let dependencies = layout.dependencies(allocated_nodes, layout.metadata())?;
			graph.layout_dependencies.push(dependencies)
		}

		Ok(graph)
	}
}

pub struct DependencyGraph<M> {
	items: Vec<crate::Item<M>>,
	ty_dependencies: Vec<Vec<crate::Item<M>>>,
	prop_dependencies: Vec<Vec<crate::Item<M>>>,
	layout_dependencies: Vec<Vec<crate::Item<M>>>,
}

impl<M> SccGraph for DependencyGraph<M> {
	type Vertex = crate::Item<M>;

	fn vertices(&self) -> &[Self::Vertex] {
		&self.items
	}

	fn successors(&self, v: Self::Vertex) -> &[Self::Vertex] {
		match v {
			crate::Item::Type(r) => &self.ty_dependencies[r.index()],
			crate::Item::Property(r) => &self.prop_dependencies[r.index()],
			crate::Item::Layout(r) => &self.layout_dependencies[r.index()],
		}
	}
}

pub struct Nodes<M> {
	nodes: BTreeMap<Id, Node<Components<M>>>,
}

impl<M: Clone> Nodes<M> {
	pub fn new(shelves: &mut Shelves<M>, nodes: BTreeMap<Id, Node<node::Components<M>>>) -> Self {
		// Step 1: allocate each type/property/layout.
		let mut allocated_nodes = BTreeMap::new();
		for (id, node) in nodes {
			let allocated_node = node.map(|components| Components {
				ty: components.ty.map_with_causes(|Meta(ty, meta)| {
					Meta(
						shelves.types.insert((id, Meta(ty, meta.clone()))).cast(),
						meta,
					)
				}),
				property: components.property.map_with_causes(|Meta(prop, meta)| {
					Meta(
						shelves
							.properties
							.insert((id, Meta(prop, meta.clone())))
							.cast(),
						meta,
					)
				}),
				layout: components.layout.map_with_causes(|Meta(layout, meta)| {
					Meta(
						shelves
							.layouts
							.insert((id, Meta(layout, meta.clone())))
							.cast(),
						meta,
					)
				}),
				layout_field: components.layout_field,
				layout_variant: components.layout_variant,
				list: components.list,
			});

			allocated_nodes.insert(id, allocated_node);
		}

		Self {
			nodes: allocated_nodes,
		}
	}

	pub fn into_model_nodes(self) -> BTreeMap<Id, treeldr::Node<M>> {
		self.nodes
			.into_iter()
			.map(|(id, node)| (id, node.into()))
			.collect()
	}

	pub fn insert(&mut self, id: Id, node: Node<Components<M>>) -> Option<Node<Components<M>>> {
		self.nodes.insert(id, node)
	}

	pub fn insert_layout(
		&mut self,
		id: Id,
		layout_ref: Ref<treeldr::layout::Definition<M>>,
		causes: M,
	) -> Option<Node<Components<M>>> {
		let node = Node::new_with(
			id,
			Components {
				layout: Meta::new(layout_ref, causes).into(),
				..Components::default()
			},
		);
		self.insert(id, node)
	}

	pub fn get(&self, id: Id) -> Option<&Node<Components<M>>> {
		self.nodes.get(&id)
	}

	pub fn require(
		&self,
		id: Id,
		expected_ty: Option<node::Type>,
		cause: &M,
	) -> Result<&Node<Components<M>>, Error<M>> {
		self.nodes
			.get(&id)
			.ok_or_else(|| Meta(error::NodeUnknown { id, expected_ty }.into(), cause.clone()))
	}

	pub fn require_type(
		&self,
		id: Id,
		cause: &M,
	) -> Result<&Meta<Ref<treeldr::ty::Definition<M>>, M>, Error<M>> {
		self.require(id, Some(node::Type::Type), cause)?
			.require_type(cause)
	}

	pub fn require_property(
		&self,
		id: Id,
		cause: &M,
	) -> Result<&Meta<Ref<treeldr::prop::Definition<M>>, M>, Error<M>> {
		self.require(id, Some(node::Type::Property), cause)?
			.require_property(cause)
	}

	pub fn require_layout(
		&self,
		id: Id,
		cause: &M,
	) -> Result<&Meta<Ref<treeldr::layout::Definition<M>>, M>, Error<M>> {
		self.require(id, Some(node::Type::Layout), cause)?
			.require_layout(cause)
	}

	pub fn require_layout_field(
		&self,
		id: Id,
		cause: &M,
	) -> Result<&Meta<layout::field::Definition<M>, M>, Error<M>> {
		self.require(id, Some(node::Type::LayoutField), cause)?
			.require_layout_field(cause)
	}

	pub fn require_layout_variant(
		&self,
		id: Id,
		cause: &M,
	) -> Result<&Meta<layout::variant::Definition<M>, M>, Error<M>> {
		self.require(id, Some(node::Type::LayoutVariant), cause)?
			.require_layout_variant(cause)
	}

	pub fn require_list(&self, id: Id, cause: &M) -> Result<ListRef<M>, Error<M>> {
		match id {
			Id::Iri(IriIndex::Iri(vocab::Term::Rdf(vocab::Rdf::Nil))) => Ok(ListRef::Nil),
			id => Ok(ListRef::Cons(
				self.require(id, Some(node::Type::List), cause)?
					.require_list(cause)?,
			)),
		}
	}
}
