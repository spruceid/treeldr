use super::{layout, list, node, prop, ty, Error, ListMut, ListRef, Node};
use crate::{error, utils::TryCollect, vocab, Caused, Id, MaybeSet, Model, Vocabulary, WithCauses};
use derivative::Derivative;
use iref::IriBuf;
use locspan::Location;
use shelves::{Ref, Shelf};
use std::collections::HashMap;

/// TreeLDR build context.
#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct Context<F> {
	/// Vocabulary.
	vocab: Vocabulary,

	/// Nodes.
	nodes: HashMap<Id, Node<node::Components<F>>>,
}

impl<F> Context<F> {
	/// Creates a new empty context.
	pub fn new() -> Self {
		Self::default()
	}

	pub fn with_vocabulary(vocab: Vocabulary) -> Self {
		Self {
			vocab,
			nodes: HashMap::new(),
		}
	}

	pub fn into_vocabulary(self) -> Vocabulary {
		self.vocab
	}

	pub fn define_native_type(
		&mut self,
		iri: IriBuf,
		native_layout: layout::Native,
		cause: Option<Location<F>>,
	) -> Result<Id, Error<F>>
	where
		F: Clone + Ord,
	{
		let id = Id::Iri(vocab::Name::from_iri(iri, self.vocabulary_mut()));
		self.declare_type(id, cause.clone());
		self.declare_layout(id, cause.clone());
		let layout = self.get_mut(id).unwrap().as_layout_mut().unwrap();
		layout.set_native(native_layout, cause.clone())?;
		layout.set_type(id, cause)?;
		Ok(id)
	}

	pub fn define_xml_types(&mut self) -> Result<(), Error<F>>
	where
		F: Clone + Ord,
	{
		self.define_native_type(
			IriBuf::new("http://www.w3.org/2001/XMLSchema#boolean").unwrap(),
			layout::Native::Boolean,
			None,
		)?;
		self.define_native_type(
			IriBuf::new("http://www.w3.org/2001/XMLSchema#int").unwrap(),
			layout::Native::Integer,
			None,
		)?;
		self.define_native_type(
			IriBuf::new("http://www.w3.org/2001/XMLSchema#integer").unwrap(),
			layout::Native::Integer,
			None,
		)?;
		self.define_native_type(
			IriBuf::new("http://www.w3.org/2001/XMLSchema#positiveInteger").unwrap(),
			layout::Native::PositiveInteger,
			None,
		)?;
		self.define_native_type(
			IriBuf::new("http://www.w3.org/2001/XMLSchema#float").unwrap(),
			layout::Native::Float,
			None,
		)?;
		self.define_native_type(
			IriBuf::new("http://www.w3.org/2001/XMLSchema#double").unwrap(),
			layout::Native::Double,
			None,
		)?;
		self.define_native_type(
			IriBuf::new("http://www.w3.org/2001/XMLSchema#string").unwrap(),
			layout::Native::String,
			None,
		)?;
		self.define_native_type(
			IriBuf::new("http://www.w3.org/2001/XMLSchema#time").unwrap(),
			layout::Native::Time,
			None,
		)?;
		self.define_native_type(
			IriBuf::new("http://www.w3.org/2001/XMLSchema#date").unwrap(),
			layout::Native::Date,
			None,
		)?;
		self.define_native_type(
			IriBuf::new("http://www.w3.org/2001/XMLSchema#dateTime").unwrap(),
			layout::Native::DateTime,
			None,
		)?;
		self.define_native_type(
			IriBuf::new("http://www.w3.org/2001/XMLSchema#anyURI").unwrap(),
			layout::Native::Uri,
			None,
		)?;

		Ok(())
	}

	/// Resolve all the reference layouts.
	///
	/// Checks that the type of a reference layout (`&T`) is equal to the type of the target layout (`T`).
	/// If no type is defined for the reference layout, it is set to the correct type.
	pub fn resolve_references(&mut self) -> Result<(), Error<F>>
	where
		F: Ord + Clone,
	{
		let mut deref_map = HashMap::new();

		for (id, node) in &self.nodes {
			if let Some(layout) = node.as_layout() {
				if let Some(desc) = layout.description() {
					if let layout::Description::Reference(target_layout_id) = desc.inner() {
						deref_map.insert(
							*id,
							Caused::new(*target_layout_id, desc.causes().preferred().cloned()),
						);
					}
				}
			}
		}

		// Assign a depth to each reference.
		// The depth correspond the the reference nesting level (`&&&&T` => 4 nesting level => depth 3).
		// References with higher depth must be resolved first.
		let mut depth_map: HashMap<_, _> = deref_map.keys().map(|id| (*id, 0)).collect();
		let mut stack: Vec<_> = deref_map.keys().map(|id| (*id, 0)).collect();
		while let Some((id, depth)) = stack.pop() {
			let current_depth = depth_map[&id];
			if depth > current_depth {
				if current_depth > 0 {
					panic!("cycling reference")
				}

				depth_map.insert(id, depth);
				if let Some(target_layout_id) = deref_map.get(&id) {
					stack.push((*target_layout_id.inner(), depth + 1));
				}
			}
		}

		// Sort references by depth (highest first).
		let mut by_depth: Vec<_> = deref_map.into_iter().collect();
		by_depth.sort_by(|(a, _), (b, _)| depth_map[b].cmp(&depth_map[a]));

		// Actually resolve the references.
		for (id, target_layout_id) in by_depth {
			let (target_layout_id, cause) = target_layout_id.into_parts();
			let target_layout = self.require_layout(target_layout_id, cause.clone())?;
			let (target_ty_id, ty_cause) = target_layout.require_ty(cause)?.clone().into_parts();
			self.get_mut(id)
				.unwrap()
				.as_layout_mut()
				.unwrap()
				.set_type(target_ty_id, ty_cause.into_preferred())?
		}

		Ok(())
	}

	pub fn build(mut self) -> Result<Model<F>, (Error<F>, Vocabulary)>
	where
		F: Ord + Clone,
	{
		if let Err(e) = self.resolve_references() {
			return Err((e, self.into_vocabulary()));
		}

		let mut allocated_shelves = AllocatedShelves::default();
		let allocated_nodes = AllocatedNodes::new(&mut allocated_shelves, self.nodes);

		match allocated_shelves
			.types
			.into_storage()
			.into_iter()
			.map(|(id, ty)| ty.build(id, &allocated_nodes))
			.try_collect()
		{
			Ok(types) => {
				match allocated_shelves
					.properties
					.into_storage()
					.into_iter()
					.map(|(id, prop)| prop.build(id, &allocated_nodes))
					.try_collect()
				{
					Ok(properties) => {
						match allocated_shelves
							.layouts
							.into_storage()
							.into_iter()
							.map(|(id, layout)| layout.build(id, &self.vocab, &allocated_nodes))
							.try_collect()
						{
							Ok(layouts) => Ok(Model::from_parts(
								self.vocab,
								allocated_nodes.into_model_nodes(),
								Shelf::new(types),
								Shelf::new(properties),
								Shelf::new(layouts),
							)),
							Err(e) => Err((e, self.vocab)),
						}
					}
					Err(e) => Err((e, self.vocab)),
				}
			}
			Err(e) => Err((e, self.vocab)),
		}
	}

	/// Returns a reference to the vocabulary.
	pub fn vocabulary(&self) -> &Vocabulary {
		&self.vocab
	}

	/// Returns a mutable reference to the vocabulary.
	pub fn vocabulary_mut(&mut self) -> &mut Vocabulary {
		&mut self.vocab
	}

	/// Returns the node associated to the given `Id`, if any.
	pub fn get(&self, id: Id) -> Option<&Node<node::Components<F>>> {
		self.nodes.get(&id)
	}

	/// Returns a mutable reference to the node associated to the given `Id`, if any.
	pub fn get_mut(&mut self, id: Id) -> Option<&mut Node<node::Components<F>>> {
		self.nodes.get_mut(&id)
	}

	pub fn nodes(&self) -> impl Iterator<Item = (Id, &Node<node::Components<F>>)> {
		self.nodes.iter().map(|(id, node)| (*id, node))
	}

	pub fn nodes_mut(&mut self) -> impl Iterator<Item = (Id, &mut Node<node::Components<F>>)> {
		self.nodes.iter_mut().map(|(id, node)| (*id, node))
	}

	/// Inserts the given node to the context.
	///
	/// Replaces any previous node with the same [`Node::id`].
	pub fn insert(&mut self, node: Node<node::Components<F>>) -> Option<Node<node::Components<F>>> {
		self.nodes.insert(node.id(), node)
	}

	pub fn add_label(&mut self, id: Id, label: String, _cause: Option<Location<F>>)
	where
		F: Ord,
	{
		if let Some(node) = self.nodes.get_mut(&id) {
			node.add_label(label)
		}
	}

	pub fn add_comment(&mut self, id: Id, comment: String, _cause: Option<Location<F>>)
	where
		F: Ord,
	{
		if let Some(node) = self.nodes.get_mut(&id) {
			node.documentation_mut().add(comment)
		}
	}

	/// Declare the given `id` as a type.
	pub fn declare_type(&mut self, id: Id, cause: Option<Location<F>>)
	where
		F: Ord,
	{
		match self.nodes.get_mut(&id) {
			Some(node) => node.declare_type(cause),
			None => {
				self.nodes.insert(id, Node::new_type(id, cause));
			}
		}
	}

	/// Declare the given `id` as a property.
	pub fn declare_property(&mut self, id: Id, cause: Option<Location<F>>)
	where
		F: Ord,
	{
		match self.nodes.get_mut(&id) {
			Some(node) => node.declare_property(cause),
			None => {
				self.nodes.insert(id, Node::new_property(id, cause));
			}
		}
	}

	/// Declare the given `id` as a layout.
	pub fn declare_layout(&mut self, id: Id, cause: Option<Location<F>>)
	where
		F: Ord,
	{
		match self.nodes.get_mut(&id) {
			Some(node) => node.declare_layout(cause),
			None => {
				self.nodes.insert(id, Node::new_layout(id, cause));
			}
		}
	}

	/// Declare the given `id` as a layout.
	pub fn declare_layout_field(&mut self, id: Id, cause: Option<Location<F>>)
	where
		F: Ord,
	{
		match self.nodes.get_mut(&id) {
			Some(node) => node.declare_layout_field(cause),
			None => {
				self.nodes.insert(id, Node::new_layout_field(id, cause));
			}
		}
	}

	/// Declare the given `id` as a list.
	pub fn declare_list(&mut self, id: Id, cause: Option<Location<F>>)
	where
		F: Ord,
	{
		match id {
			Id::Iri(vocab::Name::Rdf(vocab::Rdf::Nil)) => (),
			id => match self.nodes.get_mut(&id) {
				Some(node) => node.declare_list(cause),
				None => {
					self.nodes.insert(id, Node::new_list(id, cause));
				}
			},
		}
	}

	pub fn require_mut(
		&mut self,
		id: Id,
		cause: Option<Location<F>>,
	) -> Result<&mut Node<node::Components<F>>, Error<F>>
	where
		F: Clone,
	{
		match self.get_mut(id) {
			Some(node) => Ok(node),
			None => Err(Caused::new(
				error::NodeUnknown {
					id,
					expected_ty: None,
				}
				.into(),
				cause,
			)),
		}
	}

	pub fn require_type_mut(
		&mut self,
		id: Id,
		cause: Option<Location<F>>,
	) -> Result<&mut WithCauses<ty::Definition<F>, F>, Error<F>>
	where
		F: Clone,
	{
		match self.get_mut(id) {
			Some(node) => node.require_type_mut(cause),
			None => Err(Caused::new(
				error::NodeUnknown {
					id,
					expected_ty: Some(node::Type::Type),
				}
				.into(),
				cause,
			)),
		}
	}

	pub fn require_property_mut(
		&mut self,
		id: Id,
		cause: Option<Location<F>>,
	) -> Result<&mut WithCauses<prop::Definition<F>, F>, Error<F>>
	where
		F: Clone,
	{
		match self.get_mut(id) {
			Some(node) => node.require_property_mut(cause),
			None => Err(Caused::new(
				error::NodeUnknown {
					id,
					expected_ty: Some(node::Type::Property),
				}
				.into(),
				cause,
			)),
		}
	}

	pub fn require_layout(
		&self,
		id: Id,
		cause: Option<Location<F>>,
	) -> Result<&WithCauses<layout::Definition<F>, F>, Error<F>>
	where
		F: Clone,
	{
		match self.get(id) {
			Some(node) => node.require_layout(cause),
			None => Err(Caused::new(
				error::NodeUnknown {
					id,
					expected_ty: Some(node::Type::Layout),
				}
				.into(),
				cause,
			)),
		}
	}

	pub fn require_layout_mut(
		&mut self,
		id: Id,
		cause: Option<Location<F>>,
	) -> Result<&mut WithCauses<layout::Definition<F>, F>, Error<F>>
	where
		F: Clone,
	{
		match self.get_mut(id) {
			Some(node) => node.require_layout_mut(cause),
			None => Err(Caused::new(
				error::NodeUnknown {
					id,
					expected_ty: Some(node::Type::Layout),
				}
				.into(),
				cause,
			)),
		}
	}

	pub fn require_layout_field_mut(
		&mut self,
		id: Id,
		cause: Option<Location<F>>,
	) -> Result<&mut WithCauses<layout::field::Definition<F>, F>, Error<F>>
	where
		F: Clone,
	{
		match self.get_mut(id) {
			Some(node) => node.require_layout_field_mut(cause),
			None => Err(Caused::new(
				error::NodeUnknown {
					id,
					expected_ty: Some(node::Type::LayoutField),
				}
				.into(),
				cause,
			)),
		}
	}

	pub fn require_property_or_layout_field_mut(
		&mut self,
		id: Id,
		cause: Option<Location<F>>,
	) -> Result<node::PropertyOrLayoutField<F>, Error<F>>
	where
		F: Clone,
	{
		match self.get_mut(id) {
			Some(node) => node.require_property_or_layout_field_mut(cause),
			None => Err(Caused::new(
				error::NodeUnknown {
					id,
					expected_ty: Some(node::Type::Property),
				}
				.into(),
				cause,
			)),
		}
	}

	pub fn require_list_mut(
		&mut self,
		id: Id,
		cause: Option<Location<F>>,
	) -> Result<ListMut<F>, Error<F>>
	where
		F: Clone,
	{
		match id {
			Id::Iri(vocab::Name::Rdf(vocab::Rdf::Nil)) => Ok(ListMut::Nil),
			id => match self.get_mut(id) {
				Some(node) => Ok(ListMut::Cons(node.require_list_mut(cause)?)),
				None => Err(Caused::new(
					error::NodeUnknown {
						id,
						expected_ty: Some(node::Type::List),
					}
					.into(),
					cause,
				)),
			},
		}
	}
}

pub struct AllocatedComponents<F> {
	ty: MaybeSet<Ref<crate::ty::Definition<F>>, F>,
	property: MaybeSet<Ref<crate::prop::Definition<F>>, F>,
	layout: MaybeSet<Ref<crate::layout::Definition<F>>, F>,
	layout_field: MaybeSet<layout::field::Definition<F>, F>,
	list: MaybeSet<list::Definition<F>, F>,
}

impl<F> Node<AllocatedComponents<F>> {
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
				.layout
				.causes()
				.map(|causes| causes.preferred().cloned()),
			layout_field: self
				.value()
				.layout_field
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
	) -> Result<&WithCauses<Ref<crate::ty::Definition<F>>, F>, Error<F>>
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
	) -> Result<&WithCauses<Ref<crate::prop::Definition<F>>, F>, Error<F>>
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
	) -> Result<&WithCauses<Ref<crate::layout::Definition<F>>, F>, Error<F>>
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

impl<F> From<Node<AllocatedComponents<F>>> for crate::Node<F> {
	fn from(n: Node<AllocatedComponents<F>>) -> crate::Node<F> {
		let (id, label, doc, value) = n.into_parts();

		crate::Node::from_parts(id, label, value.ty, value.property, value.layout, doc)
	}
}

#[allow(clippy::type_complexity)]
pub struct AllocatedShelves<F> {
	types: Shelf<Vec<(Id, WithCauses<ty::Definition<F>, F>)>>,
	properties: Shelf<Vec<(Id, WithCauses<prop::Definition<F>, F>)>>,
	layouts: Shelf<Vec<(Id, WithCauses<layout::Definition<F>, F>)>>,
}

impl<F> Default for AllocatedShelves<F> {
	fn default() -> Self {
		Self {
			types: Shelf::default(),
			properties: Shelf::default(),
			layouts: Shelf::default(),
		}
	}
}

pub struct AllocatedNodes<F> {
	nodes: HashMap<Id, Node<AllocatedComponents<F>>>,
}

impl<F: Clone> AllocatedNodes<F> {
	pub fn new(
		shelves: &mut AllocatedShelves<F>,
		nodes: HashMap<Id, Node<node::Components<F>>>,
	) -> Self {
		// Step 1: allocate each type/property/layout.
		let mut allocated_nodes = HashMap::new();
		for (id, node) in nodes {
			let allocated_node = node.map(|components| AllocatedComponents {
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
				list: components.list,
			});

			allocated_nodes.insert(id, allocated_node);
		}

		Self {
			nodes: allocated_nodes,
		}
	}

	pub fn into_model_nodes(self) -> HashMap<Id, crate::Node<F>> {
		self.nodes
			.into_iter()
			.map(|(id, node)| (id, node.into()))
			.collect()
	}

	pub fn get(&self, id: Id) -> Option<&Node<AllocatedComponents<F>>> {
		self.nodes.get(&id)
	}

	pub fn require(
		&self,
		id: Id,
		expected_ty: Option<node::Type>,
		cause: Option<Location<F>>,
	) -> Result<&Node<AllocatedComponents<F>>, Error<F>> {
		self.nodes
			.get(&id)
			.ok_or_else(|| Caused::new(error::NodeUnknown { id, expected_ty }.into(), cause))
	}

	pub fn require_type(
		&self,
		id: Id,
		cause: Option<Location<F>>,
	) -> Result<&WithCauses<Ref<crate::ty::Definition<F>>, F>, Error<F>> {
		self.require(id, Some(node::Type::Type), cause.clone())?
			.require_type(cause)
	}

	pub fn require_property(
		&self,
		id: Id,
		cause: Option<Location<F>>,
	) -> Result<&WithCauses<Ref<crate::prop::Definition<F>>, F>, Error<F>> {
		self.require(id, Some(node::Type::Property), cause.clone())?
			.require_property(cause)
	}

	pub fn require_layout(
		&self,
		id: Id,
		cause: Option<Location<F>>,
	) -> Result<&WithCauses<Ref<crate::layout::Definition<F>>, F>, Error<F>> {
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

	pub fn require_list(&self, id: Id, cause: Option<Location<F>>) -> Result<ListRef<F>, Error<F>> {
		match id {
			Id::Iri(vocab::Name::Rdf(vocab::Rdf::Nil)) => Ok(ListRef::Nil),
			id => Ok(ListRef::Cons(
				self.require(id, Some(node::Type::List), cause.clone())?
					.require_list(cause)?,
			)),
		}
	}
}
