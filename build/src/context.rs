use crate::{
	component,
	error::{NodeTypeInvalid, NodeUnknown},
	layout, prop, resource, Error, FunctionalPropertyValue, IriIndex, ListRef, Property,
	PropertyValues, Single,
};
use derivative::Derivative;
use locspan::{Meta, Stripped};
use rdf_types::{Generator, VocabularyMut};
use std::{
	cmp::Ordering,
	collections::{btree_map::Entry, BTreeMap, HashMap, HashSet},
};
use treeldr::{
	metadata::Merge,
	node::Type,
	ty::SubClass,
	vocab::{self, TldrVocabulary},
	BlankIdIndex, Id, Multiple, PropertyValueRef, TId, Value,
};

pub mod build;
mod initialize;

pub type Ids<'a, M> =
	std::iter::Copied<std::collections::btree_map::Keys<'a, Id, resource::Definition<M>>>;

/// TreeLDR build context.
#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct Context<M> {
	/// Nodes.
	nodes: BTreeMap<Id, resource::Definition<M>>,
}

impl<M> Context<M> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn declare(&mut self, id: Id, metadata: M) -> &mut resource::Definition<M>
	where
		M: Merge,
	{
		match self.nodes.entry(id) {
			Entry::Occupied(entry) => {
				let node = entry.into_mut();
				node.metadata_mut().merge_with(metadata);
				node
			}
			Entry::Vacant(entry) => entry.insert(resource::Definition::new(id, metadata)),
		}
	}

	pub fn declare_with(
		&mut self,
		id: Id,
		type_: impl Into<crate::Type>,
		metadata: M,
	) -> &mut resource::Definition<M>
	where
		M: Clone + Merge,
	{
		let node = self.declare(id, metadata.clone());
		node.type_mut().insert_base(Meta(type_.into(), metadata));
		node
	}

	pub fn declare_type(&mut self, id: Id, metadata: M) -> &mut resource::Definition<M>
	where
		M: Clone + Merge,
	{
		self.declare_with(id, Type::Class(None), metadata)
	}

	pub fn declare_datatype(&mut self, id: Id, metadata: M) -> &mut resource::Definition<M>
	where
		M: Clone + Merge,
	{
		self.declare_with(id, SubClass::DataType, metadata)
	}

	pub fn declare_restriction(&mut self, id: Id, metadata: M) -> &mut resource::Definition<M>
	where
		M: Clone + Merge,
	{
		self.declare_with(id, SubClass::Restriction, metadata)
	}

	pub fn declare_datatype_restriction(
		&mut self,
		id: Id,
		metadata: M,
	) -> &mut resource::Definition<M>
	where
		M: Clone + Merge,
	{
		self.declare_with(id, Type::DatatypeRestriction, metadata)
	}

	pub fn declare_property(&mut self, id: Id, metadata: M) -> &mut resource::Definition<M>
	where
		M: Clone + Merge,
	{
		self.declare_with(id, Type::Property(None), metadata)
	}

	pub fn declare_functional_property(
		&mut self,
		id: Id,
		metadata: M,
	) -> &mut resource::Definition<M>
	where
		M: Clone + Merge,
	{
		self.declare_with(id, prop::Type::FunctionalProperty, metadata)
	}

	pub fn declare_layout(&mut self, id: Id, metadata: M) -> &mut resource::Definition<M>
	where
		M: Clone + Merge,
	{
		self.declare_with(id, component::Type::Layout, metadata)
	}

	pub fn declare_primitive_layout(
		&mut self,
		primitive: layout::Primitive,
		metadata: M,
	) -> &mut resource::Definition<M>
	where
		M: Clone + Merge,
	{
		self.declare_layout(primitive.id(), metadata)
	}

	pub fn declare_layout_field(&mut self, id: Id, metadata: M) -> &mut resource::Definition<M>
	where
		M: Clone + Merge,
	{
		self.declare_with(id, component::formatted::Type::LayoutField, metadata)
	}

	pub fn declare_layout_variant(&mut self, id: Id, metadata: M) -> &mut resource::Definition<M>
	where
		M: Clone + Merge,
	{
		self.declare_with(id, component::formatted::Type::LayoutVariant, metadata)
	}

	pub fn declare_layout_restriction(
		&mut self,
		id: Id,
		metadata: M,
	) -> &mut resource::Definition<M>
	where
		M: Clone + Merge,
	{
		self.declare_with(id, Type::LayoutRestriction, metadata)
	}

	/// Checks if `b` is a subclass of `a`.
	pub(crate) fn is_subclass_of_with(
		&self,
		visited: &mut HashSet<crate::Type>,
		a: crate::Type,
		b: crate::Type,
	) -> bool {
		if visited.insert(b) {
			match self.get(b.id().id()) {
				Some(ty) => ty
					.as_type()
					.is_subclass_of_with(self, visited, ty.as_resource(), a),
				None => false,
			}
		} else {
			false
		}
	}

	/// Checks if `b` is a subclass of `a`.
	pub fn is_subclass_of(&self, a: crate::Type, b: crate::Type) -> bool {
		let mut visited = HashSet::new();
		self.is_subclass_of_with(&mut visited, a, b)
	}

	/// Checks if `b` is a subclass or equals `a`.
	pub fn is_subclass_of_or_eq(&self, a: crate::Type, b: crate::Type) -> bool {
		a == b || self.is_subclass_of(a, b)
	}

	/// Compare `a` and `b` w.r.t the subclass relation.
	///
	/// Returns [`Ordering::Greater`] if `b` is a subclass of `a`,
	/// [`Ordering::Less`] if `a` is a subclass of `b`,
	/// [`Ordering::Equal`] if `a` is equal to `b` and `None` if both classes
	/// are unrelated.
	pub fn subclass_partial_cmp(&self, a: crate::Type, b: crate::Type) -> Option<Ordering> {
		if a == b {
			Some(Ordering::Equal)
		} else if self.is_subclass_of(a, b) {
			Some(Ordering::Greater)
		} else if self.is_subclass_of(b, a) {
			Some(Ordering::Less)
		} else {
			None
		}
	}

	pub fn len(&self) -> usize {
		self.nodes.len()
	}

	pub fn is_empty(&self) -> bool {
		self.nodes.is_empty()
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
			id => self
				.get(id)
				.map(|n| ListRef::Cons(id, n.as_list(), n.metadata())),
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

pub trait MapIds {
	fn map_ids(&mut self, f: impl Fn(Id, Option<Property>) -> Id);
}

pub trait MapIdsIn {
	fn map_ids_in(&mut self, prop: Option<Property>, f: impl Fn(Id, Option<Property>) -> Id);
}

impl<M: Merge> MapIds for Context<M> {
	fn map_ids(&mut self, f: impl Fn(Id, Option<Property>) -> Id) {
		for (id, mut node) in std::mem::take(&mut self.nodes) {
			node.map_ids(&f);
			self.nodes.insert(f(id, None), node);
		}
	}
}

impl MapIdsIn for Id {
	fn map_ids_in(&mut self, prop: Option<Property>, f: impl Fn(Id, Option<Property>) -> Id) {
		*self = f(*self, prop)
	}
}

impl MapIdsIn for Value {
	fn map_ids_in(&mut self, prop: Option<Property>, f: impl Fn(Id, Option<Property>) -> Id) {
		match self {
			Self::Node(id) => *id = f(*id, prop),
			Self::Literal(_) => (),
		}
	}
}

impl<T: MapIdsIn> MapIdsIn for Stripped<T> {
	fn map_ids_in(&mut self, prop: Option<Property>, f: impl Fn(Id, Option<Property>) -> Id) {
		self.0.map_ids_in(prop, f)
	}
}

impl<T: MapIdsIn + Ord, M: Merge> MapIdsIn for Single<T, M> {
	fn map_ids_in(&mut self, prop: Option<Property>, f: impl Fn(Id, Option<Property>) -> Id) {
		for Meta(mut t, m) in std::mem::take(self) {
			t.map_ids_in(prop, &f);
			self.insert(Meta(t, m))
		}
	}
}

impl<T: MapIds + Ord, M: Merge> MapIds for Single<T, M> {
	fn map_ids(&mut self, f: impl Fn(Id, Option<Property>) -> Id) {
		for Meta(mut t, m) in std::mem::take(self) {
			t.map_ids(&f);
			self.insert(Meta(t, m))
		}
	}
}

impl<T: MapIdsIn + Ord, M> MapIdsIn for FunctionalPropertyValue<T, M> {
	fn map_ids_in(&mut self, prop: Option<Property>, f: impl Fn(Id, Option<Property>) -> Id) {
		let result = std::mem::take(self);
		*self = result.map_properties(
			|id| TId::new(f(id.id(), None)),
			|mut t| {
				t.map_ids_in(prop, &f);
				t
			},
		)
	}
}

impl<T: MapIds + Ord, M> MapIds for FunctionalPropertyValue<T, M> {
	fn map_ids(&mut self, f: impl Fn(Id, Option<Property>) -> Id) {
		let result = std::mem::take(self);
		*self = result.map_properties(
			|id| TId::new(f(id.id(), None)),
			|mut t| {
				t.map_ids(&f);
				t
			},
		)
	}
}

impl<T: MapIdsIn + Ord, M> MapIdsIn for PropertyValues<T, M> {
	fn map_ids_in(&mut self, prop: Option<Property>, f: impl Fn(Id, Option<Property>) -> Id) {
		let result = std::mem::take(self);
		*self = result.map_properties(
			|id| TId::new(f(id.id(), None)),
			|mut t| {
				t.map_ids_in(prop, &f);
				t
			},
		)
	}
}

impl<T: MapIds + Ord, M> MapIds for PropertyValues<T, M> {
	fn map_ids(&mut self, f: impl Fn(Id, Option<Property>) -> Id) {
		let result = std::mem::take(self);
		*self = result.map_properties(
			|id| TId::new(f(id.id(), None)),
			|mut t| {
				t.map_ids(&f);
				t
			},
		)
	}
}

impl<T: MapIdsIn + Ord, M: Merge> MapIdsIn for Multiple<T, M> {
	fn map_ids_in(&mut self, prop: Option<Property>, f: impl Fn(Id, Option<Property>) -> Id) {
		for Meta(mut t, m) in std::mem::take(self) {
			t.map_ids_in(prop, &f);
			self.insert(Meta(t, m))
		}
	}
}

impl<M: Merge> MapIds for HashMap<Id, M> {
	fn map_ids(&mut self, f: impl Fn(Id, Option<Property>) -> Id) {
		for (id, m) in std::mem::take(self) {
			match self.entry(f(id, None)) {
				std::collections::hash_map::Entry::Occupied(mut e) => {
					e.get_mut().merge_with(m);
				}
				std::collections::hash_map::Entry::Vacant(e) => {
					e.insert(m);
				}
			}
		}
	}
}

impl<M: Clone> Context<M> {
	pub fn require(&self, id: Id) -> Result<&resource::Definition<M>, NodeUnknown> {
		match self.get(id) {
			Some(node) => Ok(node),
			None => Err(NodeUnknown {
				id,
				expected_type: None,
			}),
		}
	}

	pub fn require_mut(&mut self, id: Id) -> Result<&mut resource::Definition<M>, NodeUnknown> {
		match self.get_mut(id) {
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

	pub fn require_type_id(&self, id: Id) -> Result<treeldr::TId<crate::Type>, RequireError<M>> {
		Ok(self.require(id)?.require_type_id(self)?)
	}

	pub fn require_datatype_id(
		&self,
		id: Id,
	) -> Result<treeldr::TId<treeldr::ty::DataType<M>>, RequireError<M>> {
		Ok(self.require(id)?.require_datatype_id(self)?)
	}

	pub fn require_property_id(
		&self,
		id: Id,
	) -> Result<treeldr::TId<treeldr::Property>, RequireError<M>> {
		Ok(self.require(id)?.require_property_id(self)?)
	}

	pub fn require_layout_id(
		&self,
		id: Id,
	) -> Result<treeldr::TId<treeldr::Layout>, RequireError<M>> {
		Ok(self.require(id)?.require_layout_id(self)?)
	}

	pub fn require_layout_field_id(
		&self,
		id: Id,
	) -> Result<treeldr::TId<treeldr::layout::Field>, RequireError<M>> {
		Ok(self.require(id)?.require_layout_field_id(self)?)
	}

	pub fn require_layout_variant_id(
		&self,
		id: Id,
	) -> Result<treeldr::TId<treeldr::layout::Variant>, RequireError<M>> {
		Ok(self.require(id)?.require_layout_variant_id(self)?)
	}

	pub fn require_layout_restriction_id(
		&self,
		id: Id,
	) -> Result<treeldr::TId<treeldr::layout::ContainerRestriction>, RequireError<M>> {
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

	pub fn create_named_option_layout(&mut self, id: Id, item_layout: Id, cause: M) -> Id {
		let layout = self.declare_layout(id, cause.clone()).as_layout_mut();
		layout.set_option(Meta(item_layout, cause));
		id
	}

	// pub fn standard_reference<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
	// 	&mut self,
	// 	vocabulary: &mut V,
	// 	generator: &mut impl Generator<V>,
	// 	deref_ty: Id,
	// 	cause: M,
	// 	deref_cause: M,
	// ) -> Id {
	// 	match self.standard_references.get(&deref_ty).cloned() {
	// 		Some(id) => id,
	// 		None => {
	// 			let id =
	// 				self.create_reference(vocabulary, generator, deref_ty, cause, deref_cause);
	// 			self.standard_references.insert(deref_ty, id);
	// 			id
	// 		}
	// 	}
	// }

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
		let layout = self.declare_layout(id, cause.clone()).as_layout_mut();
		layout.ty_mut().insert_base(Meta(target_ty, deref_cause));
		layout.set_reference(Meta(
			Id::Iri(IriIndex::Iri(vocab::Term::TreeLdr(
				vocab::TreeLdr::Primitive(treeldr::layout::Primitive::IriBuf),
			))),
			cause,
		));
		id
	}

	pub fn create_list<I: IntoIterator<Item = Meta<vocab::StrippedObject, M>>>(
		&mut self,
		vocabulary: &mut TldrVocabulary,
		generator: &mut impl Generator<TldrVocabulary>,
		list: I,
	) -> Id
	where
		I::IntoIter: DoubleEndedIterator,
	{
		let mut head = Id::Iri(IriIndex::Iri(vocab::Term::Rdf(vocab::Rdf::Nil)));

		for Meta(item, cause) in list.into_iter().rev() {
			let id = generator.next(vocabulary);

			let node = self
				.declare_with(id, Type::List, cause.clone())
				.as_list_mut();
			node.first_mut().insert_base(Meta(
				treeldr::Value::from_rdf(vocabulary, item).unwrap(),
				cause.clone(),
			));
			node.rest_mut().insert_base(Meta(head, cause));
			head = id;
		}

		head
	}

	pub fn create_list_with<I: IntoIterator, C, G>(
		&mut self,
		vocabulary: &mut TldrVocabulary,
		generator: &mut G,
		list: I,
		mut f: C,
	) -> Id
	where
		I::IntoIter: DoubleEndedIterator,
		G: Generator<TldrVocabulary>,
		C: FnMut(I::Item, &mut Self, &mut TldrVocabulary, &mut G) -> Meta<vocab::StrippedObject, M>,
	{
		let mut head = Id::Iri(IriIndex::Iri(vocab::Term::Rdf(vocab::Rdf::Nil)));

		for item in list.into_iter().rev() {
			let id = generator.next(vocabulary);
			let Meta(item, cause) = f(item, self, vocabulary, generator);

			let node = self
				.declare_with(id, Type::List, cause.clone())
				.as_list_mut();
			node.first_mut().insert_base(Meta(
				treeldr::Value::from_rdf(vocabulary, item).unwrap(),
				cause.clone(),
			));
			node.rest_mut().insert_base(Meta(head, cause));
			head = id;
		}

		head
	}

	pub fn try_create_list_with<E, I: IntoIterator, C, G>(
		&mut self,
		vocabulary: &mut TldrVocabulary,
		generator: &mut G,
		list: I,
		mut f: C,
	) -> Result<Id, E>
	where
		E: From<Error<M>>,
		I::IntoIter: DoubleEndedIterator,
		G: Generator<TldrVocabulary>,
		C: FnMut(
			I::Item,
			&mut Self,
			&mut TldrVocabulary,
			&mut G,
		) -> Result<Meta<vocab::StrippedObject, M>, E>,
	{
		let mut head = Id::Iri(IriIndex::Iri(vocab::Term::Rdf(vocab::Rdf::Nil)));

		for item in list.into_iter().rev() {
			let id = generator.next(vocabulary);
			let Meta(item, cause) = f(item, self, vocabulary, generator)?;

			let node = self
				.declare_with(id, Type::List, cause.clone())
				.as_list_mut();
			node.first_mut().insert_base(Meta(
				treeldr::Value::from_rdf(vocabulary, item).unwrap(),
				cause.clone(),
			));
			node.rest_mut().insert_base(Meta(head, cause));
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
	pub fn at_node_property(self, id: Id, property: impl Into<Property>, meta: M) -> Error<M> {
		match self {
			Self::InvalidNodeType(e) => Meta(e.for_node_binding(id, property).into(), meta),
			Self::UnknownNode(e) => Meta(e.into(), meta),
		}
	}

	pub fn at(self, meta: M) -> Error<M> {
		match self {
			Self::InvalidNodeType(e) => Meta(e.into(), meta),
			Self::UnknownNode(e) => Meta(e.into(), meta),
		}
	}
}

pub trait HasType<M> {
	type Type: Copy + Into<crate::Type>;
	type Types<'a>: 'a + IntoIterator<Item = PropertyValueRef<'a, Self::Type, M>>
	where
		Self: 'a,
		M: 'a;

	fn types(&self) -> Self::Types<'_>;

	fn type_metadata(&self, context: &Context<M>, type_: impl Into<crate::Type>) -> Option<&M> {
		let a = type_.into();
		self.types().into_iter().find_map(
			|PropertyValueRef {
			     value: Meta(b, meta),
			     ..
			 }| {
				if context.is_subclass_of_or_eq(a, (*b).into()) {
					Some(meta)
				} else {
					None
				}
			},
		)
	}

	fn has_type(&self, context: &Context<M>, type_: impl Into<crate::Type>) -> bool {
		let a = type_.into();
		self.types().into_iter().any(
			|PropertyValueRef {
			     value: Meta(b, _), ..
			 }| { context.is_subclass_of_or_eq(a, (*b).into()) },
		)
	}
}

impl<M> HasType<M> for treeldr::node::Data<M> {
	type Type = TId<crate::Type>;
	type Types<'a> = &'a PropertyValues<TId<crate::Type>, M> where Self: 'a, M: 'a;

	fn types(&self) -> Self::Types<'_> {
		&self.type_
	}
}

impl<M> HasType<M> for resource::Data<M> {
	type Type = crate::Type;
	type Types<'a> = &'a PropertyValues<crate::Type, M> where Self: 'a, M: 'a;

	fn types(&self) -> Self::Types<'_> {
		&self.type_
	}
}
