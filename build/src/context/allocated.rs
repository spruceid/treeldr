use crate::{error, layout, list, node, prop, ty, utils::SccGraph, Error, ListRef, Node};
use derivative::Derivative;
use locspan::Location;
use shelves::{Ref, Shelf};
use std::collections::{BTreeMap, HashMap};
use treeldr::{vocab, Caused, Causes, Id, MaybeSet, WithCauses};

#[derive(Clone, Derivative)]
#[derivative(Default(bound = ""))]
pub struct Components<F> {
	ty: MaybeSet<Ref<treeldr::ty::Definition<F>>, F>,
	property: MaybeSet<Ref<treeldr::prop::Definition<F>>, F>,
	layout: MaybeSet<Ref<treeldr::layout::Definition<F>>, F>,
	layout_field: MaybeSet<layout::field::Definition<F>, F>,
	layout_variant: MaybeSet<layout::variant::Definition<F>, F>,
	list: MaybeSet<list::Definition<F>, F>,
}

impl<F> Node<Components<F>> {
	pub fn caused_types(&self) -> node::CausedTypes<F>
	where
		F: Clone,
	{
		node::CausedTypes {
			ty: self
				.value()
				.ty
				.causes()
				.map(|causes| causes.preferred().cloned()),
			property: self
				.value()
				.property
				.causes()
				.map(|causes| causes.preferred().cloned()),
			layout: self
				.value()
				.property
				.causes()
				.map(|causes| causes.preferred().cloned()),
			layout_field: self
				.value()
				.layout_field
				.causes()
				.map(|causes| causes.preferred().cloned()),
			layout_variant: self
				.value()
				.layout_variant
				.causes()
				.map(|causes| causes.preferred().cloned()),
			list: self
				.value()
				.list
				.causes()
				.map(|causes| causes.preferred().cloned()),
		}
	}

	pub fn require_type(
		&self,
		cause: Option<Location<F>>,
	) -> Result<&WithCauses<Ref<treeldr::ty::Definition<F>>, F>, Error<F>>
	where
		F: Clone,
	{
		match self.value().ty.with_causes() {
			Some(ty) => Ok(ty),
			None => Err(Caused::new(
				error::NodeInvalidType {
					id: self.id(),
					expected: node::Type::Type,
					found: self.caused_types(),
				}
				.into(),
				cause,
			)),
		}
	}

	pub fn require_property(
		&self,
		cause: Option<Location<F>>,
	) -> Result<&WithCauses<Ref<treeldr::prop::Definition<F>>, F>, Error<F>>
	where
		F: Clone,
	{
		match self.value().property.with_causes() {
			Some(prop) => Ok(prop),
			None => Err(Caused::new(
				error::NodeInvalidType {
					id: self.id(),
					expected: node::Type::Property,
					found: self.caused_types(),
				}
				.into(),
				cause,
			)),
		}
	}

	pub fn require_layout(
		&self,
		cause: Option<Location<F>>,
	) -> Result<&WithCauses<Ref<treeldr::layout::Definition<F>>, F>, Error<F>>
	where
		F: Clone,
	{
		match self.value().layout.with_causes() {
			Some(layout) => Ok(layout),
			None => Err(Caused::new(
				error::NodeInvalidType {
					id: self.id(),
					expected: node::Type::Layout,
					found: self.caused_types(),
				}
				.into(),
				cause,
			)),
		}
	}

	pub fn require_layout_field(
		&self,
		cause: Option<Location<F>>,
	) -> Result<&WithCauses<layout::field::Definition<F>, F>, Error<F>>
	where
		F: Clone,
	{
		match self.value().layout_field.with_causes() {
			Some(field) => Ok(field),
			None => Err(Caused::new(
				error::NodeInvalidType {
					id: self.id(),
					expected: node::Type::LayoutField,
					found: self.caused_types(),
				}
				.into(),
				cause,
			)),
		}
	}

	pub fn require_layout_variant(
		&self,
		cause: Option<Location<F>>,
	) -> Result<&WithCauses<layout::variant::Definition<F>, F>, Error<F>>
	where
		F: Clone,
	{
		match self.value().layout_variant.with_causes() {
			Some(variant) => Ok(variant),
			None => Err(Caused::new(
				error::NodeInvalidType {
					id: self.id(),
					expected: node::Type::LayoutVariant,
					found: self.caused_types(),
				}
				.into(),
				cause,
			)),
		}
	}

	pub fn require_list(
		&self,
		cause: Option<Location<F>>,
	) -> Result<&WithCauses<list::Definition<F>, F>, Error<F>>
	where
		F: Clone,
	{
		match self.value().list.with_causes() {
			Some(list) => Ok(list),
			None => Err(Caused::new(
				error::NodeInvalidType {
					id: self.id(),
					expected: node::Type::List,
					found: self.caused_types(),
				}
				.into(),
				cause,
			)),
		}
	}
}

impl<F> From<Node<Components<F>>> for treeldr::Node<F> {
	fn from(n: Node<Components<F>>) -> treeldr::Node<F> {
		let (id, label, doc, value) = n.into_parts();

		treeldr::Node::from_parts(id, label, value.ty, value.property, value.layout, doc)
	}
}

#[allow(clippy::type_complexity)]
pub struct Shelves<F> {
	pub types: Shelf<Vec<(Id, WithCauses<ty::Definition<F>, F>)>>,
	pub properties: Shelf<Vec<(Id, WithCauses<prop::Definition<F>, F>)>>,
	pub layouts: Shelf<Vec<(Id, WithCauses<layout::Definition<F>, F>)>>,
}

impl<F> Default for Shelves<F> {
	fn default() -> Self {
		Self {
			types: Shelf::default(),
			properties: Shelf::default(),
			layouts: Shelf::default(),
		}
	}
}

impl<F: Clone + Ord> Shelves<F> {
	pub fn dependency_graph(
		&self,
		allocated_nodes: &Nodes<F>,
	) -> Result<DependencyGraph<F>, Error<F>> {
		let mut graph: DependencyGraph<F> = DependencyGraph {
			items: Vec::with_capacity(
				self.layouts.len() + self.properties.len() + self.types.len(),
			),
			ty_dependencies: Vec::with_capacity(self.types.len()),
			prop_dependencies: Vec::with_capacity(self.properties.len()),
			layout_dependencies: Vec::with_capacity(self.layouts.len()),
		};

		for (ty_ref, (_, ty)) in &self.types {
			graph.items.push(crate::Item::Type(ty_ref.cast()));
			let dependencies = ty.dependencies(allocated_nodes, ty.causes())?;
			graph.ty_dependencies.push(dependencies)
		}

		for (prop_ref, (_, prop)) in &self.properties {
			graph.items.push(crate::Item::Property(prop_ref.cast()));
			let dependencies = prop.dependencies(allocated_nodes, prop.causes())?;
			graph.prop_dependencies.push(dependencies)
		}

		for (layout_ref, (_, layout)) in &self.layouts {
			graph.items.push(crate::Item::Layout(layout_ref.cast()));
			let dependencies = layout.dependencies(allocated_nodes, layout.causes())?;
			graph.layout_dependencies.push(dependencies)
		}

		Ok(graph)
	}
}

pub struct DependencyGraph<F> {
	items: Vec<crate::Item<F>>,
	ty_dependencies: Vec<Vec<crate::Item<F>>>,
	prop_dependencies: Vec<Vec<crate::Item<F>>>,
	layout_dependencies: Vec<Vec<crate::Item<F>>>,
}

impl<F> SccGraph for DependencyGraph<F> {
	type Vertex = crate::Item<F>;

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

pub struct Nodes<F> {
	nodes: BTreeMap<Id, Node<Components<F>>>,
}

impl<F: Clone + Ord> Nodes<F> {
	pub fn new(shelves: &mut Shelves<F>, nodes: BTreeMap<Id, Node<node::Components<F>>>) -> Self {
		// Step 1: allocate each type/property/layout.
		let mut allocated_nodes = BTreeMap::new();
		for (id, node) in nodes {
			let allocated_node = node.map(|components| Components {
				ty: components
					.ty
					.map_with_causes(|ty| shelves.types.insert((id, ty)).cast()),
				property: components
					.property
					.map_with_causes(|prop| shelves.properties.insert((id, prop)).cast()),
				layout: components
					.layout
					.map_with_causes(|layout| shelves.layouts.insert((id, layout)).cast()),
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

	pub fn into_model_nodes(self) -> HashMap<Id, treeldr::Node<F>> {
		self.nodes
			.into_iter()
			.map(|(id, node)| (id, node.into()))
			.collect()
	}

	pub fn insert(&mut self, id: Id, node: Node<Components<F>>) -> Option<Node<Components<F>>> {
		self.nodes.insert(id, node)
	}

	pub fn insert_layout(
		&mut self,
		id: Id,
		layout_ref: Ref<treeldr::layout::Definition<F>>,
		causes: impl Into<Causes<F>>,
	) -> Option<Node<Components<F>>> {
		let node = Node::new_with(
			id,
			Components {
				layout: WithCauses::new(layout_ref, causes).into(),
				..Components::default()
			},
		);
		self.insert(id, node)
	}

	pub fn get(&self, id: Id) -> Option<&Node<Components<F>>> {
		self.nodes.get(&id)
	}

	pub fn require(
		&self,
		id: Id,
		expected_ty: Option<node::Type>,
		cause: Option<Location<F>>,
	) -> Result<&Node<Components<F>>, Error<F>> {
		self.nodes
			.get(&id)
			.ok_or_else(|| Caused::new(error::NodeUnknown { id, expected_ty }.into(), cause))
	}

	pub fn require_type(
		&self,
		id: Id,
		cause: Option<Location<F>>,
	) -> Result<&WithCauses<Ref<treeldr::ty::Definition<F>>, F>, Error<F>> {
		self.require(id, Some(node::Type::Type), cause.clone())?
			.require_type(cause)
	}

	pub fn require_property(
		&self,
		id: Id,
		cause: Option<Location<F>>,
	) -> Result<&WithCauses<Ref<treeldr::prop::Definition<F>>, F>, Error<F>> {
		self.require(id, Some(node::Type::Property), cause.clone())?
			.require_property(cause)
	}

	pub fn require_layout(
		&self,
		id: Id,
		cause: Option<Location<F>>,
	) -> Result<&WithCauses<Ref<treeldr::layout::Definition<F>>, F>, Error<F>> {
		self.require(id, Some(node::Type::Layout), cause.clone())?
			.require_layout(cause)
	}

	pub fn require_layout_field(
		&self,
		id: Id,
		cause: Option<Location<F>>,
	) -> Result<&WithCauses<layout::field::Definition<F>, F>, Error<F>> {
		self.require(id, Some(node::Type::LayoutField), cause.clone())?
			.require_layout_field(cause)
	}

	pub fn require_layout_variant(
		&self,
		id: Id,
		cause: Option<Location<F>>,
	) -> Result<&WithCauses<layout::variant::Definition<F>, F>, Error<F>> {
		self.require(id, Some(node::Type::LayoutVariant), cause.clone())?
			.require_layout_variant(cause)
	}

	pub fn require_list(&self, id: Id, cause: Option<Location<F>>) -> Result<ListRef<F>, Error<F>> {
		match id {
			Id::Iri(vocab::Term::Rdf(vocab::Rdf::Nil)) => Ok(ListRef::Nil),
			id => Ok(ListRef::Cons(
				self.require(id, Some(node::Type::List), cause.clone())?
					.require_list(cause)?,
			)),
		}
	}
}
