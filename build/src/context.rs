use crate::{
	error::{NodeUnknown, NodeTypeInvalid}, Error, Property, IriIndex, ListRef, component, layout, resource, prop
};
use derivative::Derivative;
use locspan::{Meta, Stripped};
use rdf_types::{Generator, VocabularyMut};
use std::collections::{BTreeMap, HashMap, btree_map::Entry};
use treeldr::{metadata::Merge, vocab, BlankIdIndex, Id, Type, ty::SubClass, Multiple};

mod initialize;
pub mod build;

pub type Ids<'a, M> = std::iter::Copied<std::collections::btree_map::Keys<'a, Id, resource::Definition<M>>>;

/// TreeLDR build context.
#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct Context<M> {
	/// Nodes.
	nodes: BTreeMap<Id, resource::Definition<M>>,

	standard_references: HashMap<Id, Id>,
}

impl<M> Context<M> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn declare(&mut self, id: Id, metadata: M) -> &mut resource::Definition<M> where M: Merge {
		match self.nodes.entry(id) {
			Entry::Occupied(entry) => {
				let node = entry.into_mut();
				node.metadata_mut().merge_with(metadata);
				node
			},
			Entry::Vacant(entry) => entry.insert(resource::Definition::new(id, metadata))
		}
	}

	pub fn declare_with(&mut self, id: Id, type_: impl Into<Type>, metadata: M) -> &mut resource::Definition<M> where M: Clone + Merge {
		let node = self.declare(id, metadata.clone());
		node.type_mut().insert(Meta(type_.into(), metadata));
		node
	}

	pub fn declare_type(&mut self, id: Id, metadata: M) -> &mut resource::Definition<M> where M: Clone + Merge {
		self.declare_with(id, Type::Class(None), metadata)
	}

	pub fn declare_datatype(&mut self, id: Id, metadata: M) -> &mut resource::Definition<M> where M: Clone + Merge {
		self.declare_with(id, SubClass::DataType, metadata)
	}

	pub fn declare_property(&mut self, id: Id, metadata: M) -> &mut resource::Definition<M> where M: Clone + Merge {
		self.declare_with(id, Type::Property(None), metadata)
	}

	pub fn declare_functional_property(&mut self, id: Id, metadata: M) -> &mut resource::Definition<M> where M: Clone + Merge {
		self.declare_with(id, prop::Type::FunctionalProperty, metadata)
	}

	pub fn declare_layout(&mut self, id: Id, metadata: M) -> &mut resource::Definition<M> where M: Clone + Merge {
		self.declare_with(id, component::Type::Layout, metadata)
	}

	pub fn declare_primitive_layout(&mut self, primitive: layout::Primitive, metadata: M) -> &mut resource::Definition<M> where M: Clone + Merge {
		self.declare_layout(primitive.id(), metadata)
	}

	pub fn declare_layout_field(&mut self, id: Id, metadata: M) -> &mut resource::Definition<M> where M: Clone + Merge {
		self.declare_with(id, component::formatted::Type::LayoutField, metadata)
	}

	pub fn declare_layout_variant(&mut self, id: Id, metadata: M) -> &mut resource::Definition<M> where M: Clone + Merge {
		self.declare_with(id, component::formatted::Type::LayoutVariant, metadata)
	}

	/// Checks if `b` is a subclass of `a`.
	pub fn is_subclass_of(&self, a: Type, b: Type) -> bool {
		b.is_subclass_of(a)
	}

	/// Checks if `b` is a subclass or equals `a`.
	pub fn is_subclass_of_or_eq(&self, a: Type, b: Type) -> bool {
		a == b || b.is_subclass_of(a)
	}

	/// Returns the node associated to the given `Id`, if any.
	pub fn get(&self, id: Id) -> Option<&resource::Definition<M>> {
		self.nodes.get(&id)
	}

	/// Returns a mutable reference to the node associated to the given `Id`, if any.
	pub fn get_mut(&mut self, id: Id) -> Option<&mut resource::Definition<M>> {
		self.nodes.get_mut(&id)
	}

	pub fn get_list(&self, id: Id) -> Option<ListRef<M>> {
		match id {
			Id::Iri(IriIndex::Iri(vocab::Term::Rdf(vocab::Rdf::Nil))) => Some(ListRef::Nil),
			id => self.get(id).map(|n| ListRef::Cons(id, n.as_list(), n.metadata()))
		}
	}

	pub fn nodes(&self) -> impl Iterator<Item = (Id, &resource::Definition<M>)> {
		self.nodes.iter().map(|(id, node)| (*id, node))
	}

	pub fn nodes_mut(&mut self) -> impl Iterator<Item = (Id, &mut resource::Definition<M>)> {
		self.nodes.iter_mut().map(|(id, node)| (*id, node))
	}

	pub fn ids(&self) -> Ids<M> {
		self.nodes.keys().copied()
	}

	/// Inserts the given node to the context.
	///
	/// Replaces any previous node with the same [`Node::id`].
	pub fn insert(&mut self, node: resource::Definition<M>) -> Option<resource::Definition<M>> {
		self.nodes.insert(node.id(), node)
	}
}

impl<M: Clone> Context<M> {
	pub fn require(
		&self,
		id: Id
	) -> Result<&resource::Definition<M>, NodeUnknown> {
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
				Ok(ListRef::Cons(id, node.require_list(self)?, node.metadata()))
			}
		}
	}

	pub fn require_type_id(&self, id: Id) -> Result<treeldr::TId<Type>, RequireError<M>> {
		Ok(self.require(id)?.require_type_id(self)?)
	}

	pub fn require_property_id(&self, id: Id) -> Result<treeldr::TId<treeldr::Property>, RequireError<M>> {
		Ok(self.require(id)?.require_property_id(self)?)
	}

	pub fn require_layout_id(&self, id: Id) -> Result<treeldr::TId<treeldr::Layout>, RequireError<M>> {
		Ok(self.require(id)?.require_layout_id(self)?)
	}

	pub fn require_layout_field_id(&self, id: Id) -> Result<treeldr::TId<treeldr::layout::Field>, RequireError<M>> {
		Ok(self.require(id)?.require_layout_field_id(self)?)
	}

	pub fn require_layout_variant_id(&self, id: Id) -> Result<treeldr::TId<treeldr::layout::Variant>, RequireError<M>> {
		Ok(self.require(id)?.require_layout_variant_id(self)?)
	}

	pub fn require_layout_restriction_id(&self, id: Id) -> Result<treeldr::TId<treeldr::layout::Restriction>, RequireError<M>> {
		Ok(self.require(id)?.require_layout_restriction_id(self)?)
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
		let layout = self.declare_with(id, component::Type::Layout, cause.clone()).as_layout_mut();
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

			let node = self.declare_with(id, Type::List, cause.clone()).as_list_mut();
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

			let node = self.declare_with(id, Type::List, cause.clone()).as_list_mut();
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

			let node = self.declare_with(id, Type::List, cause.clone()).as_list_mut();
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
		property: impl Into<Property>,
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
	fn types(&self) -> &Multiple<Type, M>;

	fn type_metadata(&self, context: &Context<M>, type_: impl Into<Type>) -> Option<&M> {
		let a = type_.into();
		self.types().iter().find_map(|Meta(b, meta)| {
			if context.is_subclass_of_or_eq(a, *b) {
				Some(meta)
			} else {
				None
			}
		})
	}

	fn has_type(&self, context: &Context<M>, type_: impl Into<Type>) -> bool {
		let a = type_.into();
		self.types().iter().any(|Meta(b, _)| context.is_subclass_of_or_eq(a, *b))
	}
}

impl<M> HasType<M> for treeldr::node::Data<M> {
	fn types(&self) -> &Multiple<Type, M> {
		&self.type_
	}
}

impl<M> HasType<M> for resource::Data<M> {
	fn types(&self) -> &Multiple<Type, M> {
		&self.type_
	}
}