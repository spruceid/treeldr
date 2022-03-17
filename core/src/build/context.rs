use iref::{Iri, IriBuf};
use locspan::Location;
use std::collections::HashMap;
use crate::{Id, vocab, Vocabulary, Caused, WithCauses, Model, MaybeSet, utils::TryCollect};
use super::{list, ListRef, ListMut, ty, prop, layout, node, Node, Error};
use shelves::{Ref, Shelf};

/// TreeLDR build context.
pub struct Context<F> {
	/// Base IRI.
	base_iri: IriBuf,

	/// Vocabulary.
	vocab: Vocabulary,

	/// Nodes.
	nodes: HashMap<Id, Node<node::Components<F>>>
}

impl<F> Context<F> {
	/// Creates a new empty context.
	pub fn new(base_iri: impl Into<IriBuf>) -> Self {
		Self {
			base_iri: base_iri.into(),
			vocab: Vocabulary::new(),
			nodes: HashMap::new()
		}
	}

	pub fn define_native_type(
		&mut self,
		iri: IriBuf,
		native_layout: layout::Native,
		cause: Option<Location<F>>,
	) -> Result<Id, Caused<layout::Mismatch<F>, F>> where F: Clone + Ord {
		let id = Id::Iri(vocab::Name::from_iri(iri, self.vocabulary_mut()));
		self.declare_type(id, cause.clone());
		self.declare_layout(id, cause.clone());
		self.get_mut(id).unwrap().as_layout_mut().unwrap().set_native(native_layout, cause)?;
		Ok(id)
	}

	pub fn define_xml_types(&mut self) -> Result<(), Caused<layout::Mismatch<F>, F>> where F: Clone + Ord {
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

	pub fn build_model(self) -> Result<Model<F>, Caused<Error<F>, F>> where F: Ord + Clone {
		// Step 1: allocate each type/property/layout.
		let mut allocated_shelves = AllocatedShelves::default();
		let mut allocated_nodes = AllocatedNodes::new(&mut allocated_shelves, self.nodes);

		let types: Vec<_> = allocated_shelves.types.into_storage().into_iter().map(|(id, ty)| ty.build(id, &allocated_nodes)).try_collect()?;
		let properties: Vec<_> = allocated_shelves.properties.into_storage().into_iter().map(|(id, prop)| prop.build(id, &allocated_nodes)).try_collect()?;
		let layouts: Vec<_> = allocated_shelves.layouts.into_storage().into_iter().map(|(id, layout)| layout.build(id, &self.vocab, &allocated_nodes)).try_collect()?;

		let mut model = Model::with_vocabulary(self.base_iri, self.vocab);

		Ok(model)
	}

	/// Returns the current base IRI.
	pub fn base_iri(&self) -> Iri {
		self.base_iri.as_iri()
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

	pub fn nodes(&self) -> impl Iterator<Item=(Id, &Node<node::Components<F>>)> {
		self.nodes.iter().map(|(id, node)| (*id, node))
	}

	pub fn nodes_mut(&mut self) -> impl Iterator<Item=(Id, &mut Node<node::Components<F>>)> {
		self.nodes.iter_mut().map(|(id, node)| (*id, node))
	}

	/// Inserts the given node to the context.
	///
	/// Replaces any previous node with the same [`Node::id`].
	pub fn insert(&mut self, node: Node<node::Components<F>>) -> Option<Node<node::Components<F>>> {
		self.nodes.insert(node.id(), node)
	}

	pub fn add_comment(&mut self, id: Id, comment: String, _cause: Option<Location<F>>) where F: Ord {
		if let Some(node) = self.nodes.get_mut(&id) {
			node.documentation_mut().add(comment)
		}
	}

	/// Declare the given `id` as a type.
	pub fn declare_type(&mut self, id: Id, cause: Option<Location<F>>) where F: Ord {
		match self.nodes.get_mut(&id) {
			Some(node) => node.declare_type(cause),
			None => { self.nodes.insert(id, Node::new_type(id, cause)); }
		}
	}

	/// Declare the given `id` as a property.
	pub fn declare_property(&mut self, id: Id, cause: Option<Location<F>>) where F: Ord {
		match self.nodes.get_mut(&id) {
			Some(node) => node.declare_property(cause),
			None => { self.nodes.insert(id, Node::new_property(id, cause)); }
		}
	}

	/// Declare the given `id` as a layout.
	pub fn declare_layout(&mut self, id: Id, cause: Option<Location<F>>) where F: Ord {
		match self.nodes.get_mut(&id) {
			Some(node) => node.declare_layout(cause),
			None => { self.nodes.insert(id, Node::new_layout(id, cause)); }
		}
	}

	/// Declare the given `id` as a layout.
	pub fn declare_layout_field(&mut self, id: Id, cause: Option<Location<F>>) where F: Ord {
		match self.nodes.get_mut(&id) {
			Some(node) => node.declare_layout_field(cause),
			None => { self.nodes.insert(id, Node::new_layout_field(id, cause)); }
		}
	}

	/// Declare the given `id` as a list.
	pub fn declare_list(&mut self, id: Id, cause: Option<Location<F>>) where F: Ord {
		match id {
			Id::Iri(vocab::Name::Rdf(vocab::Rdf::Nil)) => (),
			id => match self.nodes.get_mut(&id) {
				Some(node) => node.declare_list(cause),
				None => { self.nodes.insert(id, Node::new_list(id, cause)); }
			}
		}
	}

	pub fn require_mut(&mut self, id: Id, cause: Option<Location<F>>) -> Result<&mut Node<node::Components<F>>, Caused<Error<F>, F>> where F: Clone {
		match self.get_mut(id) {
			Some(node) => Ok(node),
			None => Err(Caused::new(
				Error::UnknownNode {
					id,
					expected_ty: None
				},
				cause
			))
		}
	}

	pub fn require_type_mut(&mut self, id: Id, cause: Option<Location<F>>) -> Result<&mut WithCauses<ty::Definition<F>, F>, Caused<Error<F>, F>> where F: Clone {
		match self.get_mut(id) {
			Some(node) => node.require_type_mut(cause),
			None => Err(Caused::new(
				Error::UnknownNode {
					id,
					expected_ty: Some(node::Type::Type)
				},
				cause
			))
		}
	}

	pub fn require_property_mut(&mut self, id: Id, cause: Option<Location<F>>) -> Result<&mut WithCauses<prop::Definition<F>, F>, Caused<Error<F>, F>> where F: Clone {
		match self.get_mut(id) {
			Some(node) => node.require_property_mut(cause),
			None => Err(Caused::new(
				Error::UnknownNode {
					id,
					expected_ty: Some(node::Type::Property)
				},
				cause
			))
		}
	}

	pub fn require_layout_mut(&mut self, id: Id, cause: Option<Location<F>>) -> Result<&mut WithCauses<layout::Definition<F>, F>, Caused<Error<F>, F>> where F: Clone {
		match self.get_mut(id) {
			Some(node) => node.require_layout_mut(cause),
			None => Err(Caused::new(
				Error::UnknownNode {
					id,
					expected_ty: Some(node::Type::Layout)
				},
				cause
			))
		}
	}

	pub fn require_layout_field_mut(&mut self, id: Id, cause: Option<Location<F>>) -> Result<&mut WithCauses<layout::field::Definition<F>, F>, Caused<Error<F>, F>> where F: Clone {
		match self.get_mut(id) {
			Some(node) => node.require_layout_field_mut(cause),
			None => Err(Caused::new(
				Error::UnknownNode {
					id,
					expected_ty: Some(node::Type::LayoutField)
				},
				cause
			))
		}
	}

	pub fn require_property_or_layout_field_mut(&mut self, id: Id, cause: Option<Location<F>>) -> Result<(Option<&mut WithCauses<prop::Definition<F>, F>>, Option<&mut WithCauses<layout::field::Definition<F>, F>>), Caused<Error<F>, F>> where F: Clone {
		match self.get_mut(id) {
			Some(node) => node.require_property_or_layout_field_mut(cause),
			None => Err(Caused::new(
				Error::UnknownNode {
					id,
					expected_ty: Some(node::Type::Property)
				},
				cause
			))
		}
	}

	pub fn require_list_mut(&mut self, id: Id, cause: Option<Location<F>>) -> Result<ListMut<F>, Caused<Error<F>, F>> where F: Clone {
		match id {
			Id::Iri(vocab::Name::Rdf(vocab::Rdf::Nil)) => Ok(ListMut::Nil),
			id => match self.get_mut(id) {
				Some(node) => Ok(ListMut::Cons(node.require_list_mut(cause)?)),
				None => Err(Caused::new(
					Error::UnknownNode {
						id,
						expected_ty: Some(node::Type::List)
					},
					cause
				))
			}
		}
	}
}

pub struct AllocatedComponents<F> {
	ty: MaybeSet<Ref<crate::ty::Definition<F>>, F>,
	property: MaybeSet<Ref<crate::prop::Definition<F>>, F>,
	layout: MaybeSet<Ref<crate::layout::Definition<F>>, F>,
	layout_field: MaybeSet<layout::field::Definition<F>, F>,
	list: MaybeSet<list::Definition<F>, F>
}

impl<F> Node<AllocatedComponents<F>> {
	pub fn caused_types(&self) -> node::CausedTypes<F> where F: Clone {
		node::CausedTypes {
			ty: self.value().ty.causes().map(|causes| causes.preferred().cloned()),
			property: self.value().property.causes().map(|causes| causes.preferred().cloned()),
			layout: self.value().layout.causes().map(|causes| causes.preferred().cloned()),
			layout_field: self.value().layout_field.causes().map(|causes| causes.preferred().cloned()),
			list: self.value().list.causes().map(|causes| causes.preferred().cloned()),
		}
	}

	pub fn require_type(&self, cause: Option<Location<F>>) -> Result<&WithCauses<Ref<crate::ty::Definition<F>>, F>, Caused<Error<F>, F>> where F: Clone {
		match self.value().ty.with_causes() {
			Some(ty) => Ok(ty),
			None => Err(Caused::new(
				Error::InvalidNodeType {
					id: self.id(),
					expected: node::Type::Type,
					found: self.caused_types()
				},
				cause
			))
		}
	}

	pub fn require_property(&self, cause: Option<Location<F>>) -> Result<&WithCauses<Ref<crate::prop::Definition<F>>, F>, Caused<Error<F>, F>> where F: Clone {
		match self.value().property.with_causes() {
			Some(prop) => Ok(prop),
			None => Err(Caused::new(
				Error::InvalidNodeType {
					id: self.id(),
					expected: node::Type::Property,
					found: self.caused_types()
				},
				cause
			))
		}
	}

	pub fn require_layout(&self, cause: Option<Location<F>>) -> Result<&WithCauses<Ref<crate::layout::Definition<F>>, F>, Caused<Error<F>, F>> where F: Clone {
		match self.value().layout.with_causes() {
			Some(layout) => Ok(layout),
			None => Err(Caused::new(
				Error::InvalidNodeType {
					id: self.id(),
					expected: node::Type::Layout,
					found: self.caused_types()
				},
				cause
			))
		}
	}

	pub fn require_layout_field(&self, cause: Option<Location<F>>) -> Result<&WithCauses<layout::field::Definition<F>, F>, Caused<Error<F>, F>> where F: Clone {
		match self.value().layout_field.with_causes() {
			Some(field) => Ok(field),
			None => Err(Caused::new(
				Error::InvalidNodeType {
					id: self.id(),
					expected: node::Type::LayoutField,
					found: self.caused_types()
				},
				cause
			))
		}
	}

	pub fn require_list(&self, cause: Option<Location<F>>) -> Result<&WithCauses<list::Definition<F>, F>, Caused<Error<F>, F>> where F: Clone {
		match self.value().list.with_causes() {
			Some(list) => Ok(list),
			None => Err(Caused::new(
				Error::InvalidNodeType {
					id: self.id(),
					expected: node::Type::List,
					found: self.caused_types()
				},
				cause
			))
		}
	}
}

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
			layouts: Shelf::default()
		}
	}
}

pub struct AllocatedNodes<F> {
	nodes: HashMap<Id, Node<AllocatedComponents<F>>>
}

impl<F: Clone> AllocatedNodes<F> {
	pub fn new(shelves: &mut AllocatedShelves<F>, nodes: HashMap<Id, Node<node::Components<F>>>) -> Self {
		// Step 1: allocate each type/property/layout.
		let mut allocated_nodes = HashMap::new();
		for (id, node) in nodes {
			let allocated_node = node.map(|components| AllocatedComponents {
				ty: components.ty.map_with_causes(|ty| shelves.types.insert((id, ty)).cast()),
				property: components.property.map_with_causes(|prop| shelves.properties.insert((id, prop)).cast()),
				layout: components.layout.map_with_causes(|layout| shelves.layouts.insert((id, layout)).cast()),
				layout_field: components.layout_field,
				list: components.list
			});

			allocated_nodes.insert(id, allocated_node);
		}

		Self {
			nodes: allocated_nodes
		}
	}

	pub fn require(&self, id: Id, expected_ty: Option<node::Type>, cause: Option<Location<F>>) -> Result<&Node<AllocatedComponents<F>>, Caused<Error<F>, F>> {
		self.nodes.get(&id).ok_or_else(|| Caused::new(
			Error::UnknownNode {
				id,
				expected_ty
			},
			cause
		))
	}

	pub fn require_type(&self, id: Id, cause: Option<Location<F>>) -> Result<&WithCauses<Ref<crate::ty::Definition<F>>, F>, Caused<Error<F>, F>> {
		self.require(id, Some(node::Type::Type), cause.clone())?.require_type(cause)
	}

	pub fn require_property(&self, id: Id, cause: Option<Location<F>>) -> Result<&WithCauses<Ref<crate::prop::Definition<F>>, F>, Caused<Error<F>, F>> {
		self.require(id, Some(node::Type::Property), cause.clone())?.require_property(cause)
	}

	pub fn require_layout(&self, id: Id, cause: Option<Location<F>>) -> Result<&WithCauses<Ref<crate::layout::Definition<F>>, F>, Caused<Error<F>, F>> {
		self.require(id, Some(node::Type::Layout), cause.clone())?.require_layout(cause)
	}

	pub fn require_layout_field(&self, id: Id, cause: Option<Location<F>>) -> Result<&WithCauses<layout::field::Definition<F>, F>, Caused<Error<F>, F>> {
		self.require(id, Some(node::Type::LayoutField), cause.clone())?.require_layout_field(cause)
	}

	pub fn require_list(&self, id: Id, cause: Option<Location<F>>) -> Result<ListRef<F>, Caused<Error<F>, F>> {
		match id {
			Id::Iri(vocab::Name::Rdf(vocab::Rdf::Nil)) => Ok(ListRef::Nil),
			id => {
				Ok(ListRef::Cons(self.require(id, Some(node::Type::List), cause.clone())?.require_list(cause)?))
			}
		}
	}
}