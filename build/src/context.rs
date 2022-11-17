use crate::{
	error::{NodeUnknown, NodeTypeInvalid}, node, Error, IriIndex, ListRef, Node, component
};
use derivative::Derivative;
use locspan::{Meta, Stripped};
use rdf_types::{Generator, VocabularyMut};
use std::collections::{BTreeMap, HashMap, btree_map::Entry};
use treeldr::{metadata::Merge, vocab, BlankIdIndex, Id, Type};

mod initialize;
mod build;

/// TreeLDR build context.
#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct Context<M> {
	/// Nodes.
	nodes: BTreeMap<Id, Node<M>>,

	standard_references: HashMap<Id, Id>,
}

impl<M> Context<M> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn declare(&mut self, id: Id, metadata: M) -> &mut Node<M> where M: Merge {
		match self.nodes.entry(id) {
			Entry::Occupied(entry) => {
				let node = entry.get_mut();
				node.metadata_mut().merge_with(metadata);
				node
			},
			Entry::Vacant(entry) => entry.insert(Node::new(id, metadata))
		}
	}

	pub fn declare_with(&mut self, id: Id, type_: impl Into<Type>, metadata: M) -> &mut Node<M> where M: Clone + Merge {
		let node = self.declare(id, metadata.clone());
		node.type_mut().insert(Meta(type_.into(), metadata));
		node
	}

	/// Returns the node associated to the given `Id`, if any.
	pub fn get(&self, id: Id) -> Option<&Node<M>> {
		self.nodes.get(&id)
	}

	/// Returns a mutable reference to the node associated to the given `Id`, if any.
	pub fn get_mut(&mut self, id: Id) -> Option<&mut Node<M>> {
		self.nodes.get_mut(&id)
	}

	pub fn get_list(&self, id: Id) -> Option<ListRef<M>> {
		match id {
			Id::Iri(IriIndex::Iri(vocab::Term::Rdf(vocab::Rdf::Nil))) => Some(ListRef::Nil),
			id => self.get(id).map(|n| ListRef::Cons(id, n.as_list(), n.metadata()))
		}
	}

	pub fn nodes(&self) -> impl Iterator<Item = (Id, &Node<M>)> {
		self.nodes.iter().map(|(id, node)| (*id, node))
	}

	pub fn nodes_mut(&mut self) -> impl Iterator<Item = (Id, &mut Node<M>)> {
		self.nodes.iter_mut().map(|(id, node)| (*id, node))
	}

	/// Inserts the given node to the context.
	///
	/// Replaces any previous node with the same [`Node::id`].
	pub fn insert(&mut self, node: Node<M>) -> Option<Node<M>> {
		self.nodes.insert(node.id(), node)
	}
}

impl<M: Clone> Context<M> {
	pub fn require(
		&mut self,
		id: Id
	) -> Result<&Node<M>, NodeUnknown> {
		match self.get(id) {
			Some(node) => Ok(node),
			None => Err(NodeUnknown {
				id,
				expected_type: None,
			}),
		}
	}

	pub fn require_list(&self, id: Id) -> Result<ListRef<M>, RequireError<M>> {
		match id {
			Id::Iri(IriIndex::Iri(vocab::Term::Rdf(vocab::Rdf::Nil))) => Ok(ListRef::Nil),
			id => {
				let node = self.require(id)?;
				Ok(ListRef::Cons(id, node.require_list()?, node.metadata()))
			}
		}
	}

	pub fn require_type_id(&self, id: Id) -> Result<treeldr::TId<Type>, RequireError<M>> {
		Ok(self.require(id)?.require_type_id()?)
	}

	pub fn require_property_id(&self, id: Id) -> Result<treeldr::TId<treeldr::Property>, RequireError<M>> {
		Ok(self.require(id)?.require_property_id()?)
	}

	pub fn require_layout_id(&self, id: Id) -> Result<treeldr::TId<treeldr::Layout>, RequireError<M>> {
		Ok(self.require(id)?.require_layout_id()?)
	}

	pub fn require_layout_field_id(&self, id: Id) -> Result<treeldr::TId<treeldr::layout::Field>, RequireError<M>> {
		Ok(self.require(id)?.require_layout_field_id()?)
	}

	pub fn require_layout_variant_id(&self, id: Id) -> Result<treeldr::TId<treeldr::layout::Variant>, RequireError<M>> {
		Ok(self.require(id)?.require_layout_variant_id()?)
	}

	pub fn require_layout_restriction_id(&self, id: Id) -> Result<treeldr::TId<treeldr::layout::Restriction>, RequireError<M>> {
		Ok(self.require(id)?.require_layout_restriction_id()?)
	}
}

impl<M: Clone + Merge> Context<M> {
	pub fn create_option_layout<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		item_layout: Id,
		cause: M,
	) -> Id {
		let id = generator.next(vocabulary);
		self.create_named_option_layout(id, item_layout, cause)
	}

	pub fn create_named_option_layout(
		&mut self,
		id: Id,
		item_layout: Id,
		cause: M,
	) -> Id {
		let layout = self.declare_with(id, component::Type::Layout, cause.clone()).as_layout_mut();
		layout.set_option(Meta(item_layout, cause));
		id
	}

	pub fn standard_reference<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		deref_ty: Id,
		cause: M,
		deref_cause: M,
	) -> Id {
		match self.standard_references.get(&deref_ty).cloned() {
			Some(id) => id,
			None => {
				let id =
					self.create_reference(vocabulary, generator, deref_ty, cause, deref_cause);
				self.standard_references.insert(deref_ty, id);
				id
			}
		}
	}

	pub fn create_reference<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		target_ty: Id,
		cause: M,
		deref_cause: M,
	) -> Id {
		let id = generator.next(vocabulary);
		self.create_named_reference(id, target_ty, cause, deref_cause)
	}

	pub fn create_named_reference(
		&mut self,
		id: Id,
		target_ty: Id,
		cause: M,
		deref_cause: M,
	) -> Id {
		let layout = self.declare_with(id, component::Type::Layout, cause).as_layout_mut();
		layout.ty_mut().insert(Meta(target_ty, deref_cause));
		layout.set_reference(Meta(
			Id::Iri(IriIndex::Iri(vocab::Term::TreeLdr(
				vocab::TreeLdr::Primitive(treeldr::layout::Primitive::Iri),
			))),
			cause
		));
		id
	}

	pub fn create_list<
		I: IntoIterator<Item = Meta<vocab::Object<M>, M>>,
		V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>,
	>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		list: I,
	) -> Id
	where
		I::IntoIter: DoubleEndedIterator,
	{
		let mut head = Id::Iri(IriIndex::Iri(vocab::Term::Rdf(vocab::Rdf::Nil)));

		for Meta(item, cause) in list.into_iter().rev() {
			let id = generator.next(vocabulary);

			let node = self.declare_with(id, Type::List, cause).as_list_mut();
			node.first_mut().insert(Meta(Stripped(item), cause.clone()));
			node.rest_mut().insert(Meta(head, cause));
			head = id;
		}

		head
	}

	pub fn create_list_with<I: IntoIterator, C, V>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		list: I,
		mut f: C,
	) -> Result<Id, Error<M>>
	where
		I::IntoIter: DoubleEndedIterator,
		V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>,
		C: FnMut(I::Item, &mut Self, &mut V) -> Meta<vocab::Object<M>, M>,
	{
		let mut head = Id::Iri(IriIndex::Iri(vocab::Term::Rdf(vocab::Rdf::Nil)));

		for item in list.into_iter().rev() {
			let id = generator.next(vocabulary);
			let Meta(item, cause) = f(item, self, vocabulary);

			let node = self.declare_with(id, Type::List, cause).as_list_mut();
			node.first_mut().insert(Meta(Stripped(item), cause.clone()));
			node.rest_mut().insert(Meta(head, cause));
			head = id;
		}

		Ok(head)
	}

	pub fn try_create_list_with<E, I: IntoIterator, C, V, G>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut G,
		list: I,
		mut f: C,
	) -> Result<Id, E>
	where
		E: From<Error<M>>,
		I::IntoIter: DoubleEndedIterator,
		V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>,
		G: Generator<V>,
		C: FnMut(I::Item, &mut Self, &mut V, &mut G) -> Result<Meta<vocab::Object<M>, M>, E>,
	{
		let mut head = Id::Iri(IriIndex::Iri(vocab::Term::Rdf(vocab::Rdf::Nil)));

		for item in list.into_iter().rev() {
			let id = generator.next(vocabulary);
			let Meta(item, cause) = f(item, self, vocabulary, generator)?;

			let node = self.declare_with(id, Type::List, cause).as_list_mut();
			node.first_mut().insert(Meta(Stripped(item), cause.clone()));
			node.rest_mut().insert(Meta(head, cause));
			head = id;
		}

		Ok(head)
	}
}

pub enum RequireError<M> {
	UnknownNode(NodeUnknown),
	InvalidNodeType(NodeTypeInvalid<M>),
}

impl<M> From<NodeUnknown> for RequireError<M> {
	fn from(e: NodeUnknown) -> Self {
		Self::UnknownNode(e)
	}
}

impl<M> From<NodeTypeInvalid<M>> for RequireError<M> {
	fn from(e: NodeTypeInvalid<M>) -> Self {
		Self::InvalidNodeType(e)
	}
}

impl<M> RequireError<M> {
	pub fn at_node_property(
		self,
		id: Id,
		property: impl Into<node::Property>,
		meta: M,
	) -> Error<M> {
		match self {
			Self::InvalidNodeType(e) => Meta(e.for_node_binding(id, property).into(), meta),
			Self::UnknownNode(e) => Meta(e.into(), meta),
		}
	}

	pub fn at(
		self,
		meta: M,
	) -> Error<M> {
		match self {
			Self::InvalidNodeType(e) => Meta(e.into(), meta),
			Self::UnknownNode(e) => Meta(e.into(), meta),
		}
	}
}

pub trait HasType<M> {
	fn has_type(&self, context: &Context<M>, type_: impl Into<Type>) -> bool;
}

impl<M> HasType<M> for treeldr::node::Data<M> {
	fn has_type(&self, context: &Context<M>, type_: impl Into<Type>) -> bool {
		self.type_.contains(&type_.into())
	}
}