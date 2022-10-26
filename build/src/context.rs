use crate::{
	error, layout, node, prop, ty, Descriptions, Error, IriIndex, ListMut, ListRef, Node,
	ParentLayout, Simplify, SubLayout,
};
use derivative::Derivative;
use locspan::Meta;
use rdf_types::{Generator, Vocabulary, VocabularyMut};
use shelves::Shelf;
use std::collections::{BTreeMap, HashMap};
use treeldr::{metadata::Merge, vocab, BlankIdIndex, Id, Model};

pub mod allocated;

/// TreeLDR build context.
#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct Context<M, D: Descriptions<M> = crate::StandardDescriptions> {
	/// Nodes.
	nodes: BTreeMap<Id, Node<node::Components<M, D>>>,

	layout_relations: HashMap<Id, LayoutRelations<M>>,

	standard_references: HashMap<Id, Id>,
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
struct LayoutRelations<M> {
	sub: Vec<SubLayout<M>>,
	parent: Vec<Meta<ParentLayout, M>>,
}

impl<M, D: Descriptions<M>> Context<M, D> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn define_primitive_datatype(
		&mut self,
		id: Id,
		primitive_type: ty::data::Primitive,
		metadata: M,
	) -> Result<Id, Error<M>>
	where
		M: Clone + Merge,
	{
		self.declare_type(id, metadata.clone());
		let ty = self.get_mut(id).unwrap().as_type_mut().unwrap();
		let dt = ty.require_datatype_mut(&metadata)?;
		dt.set_primitive(primitive_type, metadata)?;
		Ok(id)
	}

	pub fn define_primitive_layout(
		&mut self,
		primitive_layout: layout::Primitive,
		metadata: M,
	) -> Result<Id, Error<M>>
	where
		M: Clone + Merge,
	{
		let id = Id::Iri(IriIndex::Iri(vocab::Term::TreeLdr(
			vocab::TreeLdr::Primitive(primitive_layout),
		)));
		self.declare_layout(id, metadata.clone());
		let layout = self.get_mut(id).unwrap().as_layout_mut().unwrap();
		layout.set_primitive(primitive_layout, metadata)?;
		Ok(id)
	}

	pub fn apply_built_in_definitions_with<
		V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>,
	>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		metadata: M,
	) -> Result<(), Error<M>>
	where
		M: Clone + Merge,
	{
		self.define_rdf_types(vocabulary, generator, metadata.clone())?;
		self.define_xsd_types(metadata.clone())?;
		self.define_treeldr_types(metadata)
	}

	pub fn apply_built_in_definitions<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<(), Error<M>>
	where
		M: Default + Clone + Merge,
	{
		self.apply_built_in_definitions_with(vocabulary, generator, M::default())
	}

	pub fn create_option_layout<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		item_layout: Id,
		cause: M,
	) -> Result<Id, Error<M>>
	where
		M: Clone + Merge,
	{
		let id = generator.next(vocabulary);
		self.create_named_option_layout(id, item_layout, cause)
	}

	pub fn create_named_option_layout(
		&mut self,
		id: Id,
		item_layout: Id,
		cause: M,
	) -> Result<Id, Error<M>>
	where
		M: Clone + Merge,
	{
		self.declare_layout(id, cause.clone());
		let layout = self.get_mut(id).unwrap().as_layout_mut().unwrap();
		layout.set_option(item_layout, cause)?;
		Ok(id)
	}

	pub fn standard_reference<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		deref_ty: Id,
		cause: M,
		deref_cause: M,
	) -> Result<Id, Error<M>>
	where
		M: Clone + Merge,
	{
		match self.standard_references.get(&deref_ty).cloned() {
			Some(id) => Ok(id),
			None => {
				let id =
					self.create_reference(vocabulary, generator, deref_ty, cause, deref_cause)?;
				self.standard_references.insert(deref_ty, id);
				Ok(id)
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
	) -> Result<Id, Error<M>>
	where
		M: Clone + Merge,
	{
		let id = generator.next(vocabulary);
		self.create_named_reference(id, target_ty, cause, deref_cause)
	}

	pub fn create_named_reference(
		&mut self,
		id: Id,
		target_ty: Id,
		cause: M,
		deref_cause: M,
	) -> Result<Id, Error<M>>
	where
		M: Clone + Merge,
	{
		self.declare_layout(id, cause.clone());
		let layout = self.get_mut(id).unwrap().as_layout_mut().unwrap();
		layout.set_type(target_ty, deref_cause)?;
		layout.set_reference(
			Id::Iri(IriIndex::Iri(vocab::Term::TreeLdr(
				vocab::TreeLdr::Primitive(treeldr::layout::Primitive::Iri),
			))),
			cause,
		)?;
		Ok(id)
	}

	pub fn define_rdf_types<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		metadata: M,
	) -> Result<(), Error<M>>
	where
		M: Clone + Merge,
	{
		use vocab::{Rdf, Rdfs, Term};
		// rdfs:Resource
		self.declare_type(
			Id::Iri(IriIndex::Iri(Term::Rdfs(Rdfs::Resource))),
			metadata.clone(),
		);
		self.declare_layout(
			Id::Iri(IriIndex::Iri(Term::Rdfs(Rdfs::Resource))),
			metadata.clone(),
		);
		let id_field = generator.next(vocabulary);
		self.declare_layout_field(id_field, metadata.clone());
		let resource_ref_layout = self.standard_reference(
			vocabulary,
			generator,
			Id::Iri(IriIndex::Iri(Term::Rdfs(Rdfs::Resource))),
			metadata.clone(),
			metadata.clone(),
		)?;
		let field_layout = self.create_option_layout(
			vocabulary,
			generator,
			resource_ref_layout,
			metadata.clone(),
		)?;
		let field = self
			.get_mut(id_field)
			.unwrap()
			.as_layout_field_mut()
			.unwrap();
		field.set_property(
			Id::Iri(IriIndex::Iri(Term::TreeLdr(vocab::TreeLdr::Self_))),
			metadata.clone(),
		)?;
		field.set_name(treeldr::Name::new("id").unwrap(), metadata.clone())?;
		field.set_layout(field_layout, metadata.clone())?;
		let fields_id = self.create_list(
			vocabulary,
			generator,
			[Meta(id_field.into_term(), metadata.clone())],
		)?;
		let layout = self
			.get_mut(Id::Iri(IriIndex::Iri(Term::Rdfs(Rdfs::Resource))))
			.unwrap()
			.as_layout_mut()
			.unwrap();
		layout.set_fields(fields_id, metadata.clone())?;

		// rdfs:Class
		self.declare_type(
			Id::Iri(IriIndex::Iri(Term::Rdfs(Rdfs::Class))),
			metadata.clone(),
		);

		// rdf:Property
		self.declare_type(
			Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::Property))),
			metadata.clone(),
		);

		// rdf:type
		self.declare_property(
			Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::Type))),
			metadata.clone(),
		);
		let prop = self
			.get_mut(Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::Type))))
			.unwrap()
			.as_property_mut()
			.unwrap();
		prop.set_domain(
			Id::Iri(IriIndex::Iri(Term::Rdfs(Rdfs::Resource))),
			metadata.clone(),
		);
		prop.set_range(
			Id::Iri(IriIndex::Iri(Term::Rdfs(Rdfs::Class))),
			metadata.clone(),
		)?;

		// rdf:List
		self.declare_type(
			Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::List))),
			metadata.clone(),
		);
		let list = self
			.get_mut(Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::List))))
			.unwrap()
			.as_type_mut()
			.unwrap();
		list.declare_property(
			Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::First))),
			metadata.clone(),
		)?;
		list.declare_property(
			Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::Rest))),
			metadata.clone(),
		)?;

		// rdf:first
		self.declare_property(
			Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::First))),
			metadata.clone(),
		);
		let prop = self
			.get_mut(Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::First))))
			.unwrap()
			.as_property_mut()
			.unwrap();
		prop.set_domain(
			Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::List))),
			metadata.clone(),
		);
		prop.set_range(
			Id::Iri(IriIndex::Iri(Term::Rdfs(Rdfs::Resource))),
			metadata.clone(),
		)?;

		// rdf:rest
		self.declare_property(
			Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::Rest))),
			metadata.clone(),
		);
		let prop = self
			.get_mut(Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::Rest))))
			.unwrap()
			.as_property_mut()
			.unwrap();
		prop.set_domain(
			Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::List))),
			metadata.clone(),
		);
		prop.set_range(Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::List))), metadata)
	}

	pub fn define_xsd_types(&mut self, metadata: M) -> Result<(), Error<M>>
	where
		M: Clone + Merge,
	{
		use vocab::{Term, Xsd};
		self.define_primitive_datatype(
			Id::Iri(IriIndex::Iri(Term::Xsd(Xsd::String))),
			ty::data::Primitive::String,
			metadata,
		)?;
		Ok(())
	}

	pub fn define_treeldr_types(&mut self, metadata: M) -> Result<(), Error<M>>
	where
		M: Clone + Merge,
	{
		use layout::Primitive;

		self.declare_property(
			Id::Iri(IriIndex::Iri(vocab::Term::TreeLdr(vocab::TreeLdr::Self_))),
			metadata.clone(),
		);
		let prop = self
			.get_mut(Id::Iri(IriIndex::Iri(vocab::Term::TreeLdr(
				vocab::TreeLdr::Self_,
			))))
			.unwrap()
			.as_property_mut()
			.unwrap();
		prop.set_range(
			Id::Iri(IriIndex::Iri(vocab::Term::Rdfs(vocab::Rdfs::Resource))),
			metadata.clone(),
		)?;

		self.define_primitive_layout(Primitive::Boolean, metadata.clone())?;
		self.define_primitive_layout(Primitive::Integer, metadata.clone())?;
		self.define_primitive_layout(Primitive::UnsignedInteger, metadata.clone())?;
		self.define_primitive_layout(Primitive::Float, metadata.clone())?;
		self.define_primitive_layout(Primitive::Double, metadata.clone())?;
		self.define_primitive_layout(Primitive::String, metadata.clone())?;
		self.define_primitive_layout(Primitive::Time, metadata.clone())?;
		self.define_primitive_layout(Primitive::Date, metadata.clone())?;
		self.define_primitive_layout(Primitive::DateTime, metadata.clone())?;
		self.define_primitive_layout(Primitive::Iri, metadata.clone())?;
		self.define_primitive_layout(Primitive::Uri, metadata.clone())?;
		self.define_primitive_layout(Primitive::Url, metadata)?;

		Ok(())
	}

	pub fn try_map<
		G: Descriptions<M>,
		E,
		V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>,
	>(
		&self,
		map: &impl crate::TryMap<M, E, D, G>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<Context<M, G>, E>
	where
		M: Clone,
	{
		let mut target = Context::new();

		for (id, node) in &self.nodes {
			let mapped_node = node
				.clone()
				.try_map(|desc| desc.try_map(map, self, &mut target, vocabulary, generator))?;
			target.nodes.insert(*id, mapped_node);
		}

		Ok(target)
	}

	pub fn simplify<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<Context<M>, <D as Simplify<M>>::Error>
	where
		D: Simplify<M>,
		M: Clone,
	{
		let map = D::TryMap::default();
		self.try_map(&map, vocabulary, generator)
	}

	/// Returns the node associated to the given `Id`, if any.
	pub fn get(&self, id: Id) -> Option<&Node<node::Components<M, D>>> {
		self.nodes.get(&id)
	}

	/// Returns a mutable reference to the node associated to the given `Id`, if any.
	pub fn get_mut(&mut self, id: Id) -> Option<&mut Node<node::Components<M, D>>> {
		self.nodes.get_mut(&id)
	}

	pub fn nodes(&self) -> impl Iterator<Item = (Id, &Node<node::Components<M, D>>)> {
		self.nodes.iter().map(|(id, node)| (*id, node))
	}

	pub fn nodes_mut(&mut self) -> impl Iterator<Item = (Id, &mut Node<node::Components<M, D>>)> {
		self.nodes.iter_mut().map(|(id, node)| (*id, node))
	}

	/// Inserts the given node to the context.
	///
	/// Replaces any previous node with the same [`Node::id`].
	pub fn insert(
		&mut self,
		node: Node<node::Components<M, D>>,
	) -> Option<Node<node::Components<M, D>>> {
		self.nodes.insert(node.id(), node)
	}

	pub fn add_label(&mut self, id: Id, label: String, _cause: M) {
		if let Some(node) = self.nodes.get_mut(&id) {
			node.add_label(label)
		}
	}

	pub fn add_comment(&mut self, id: Id, comment: String, _cause: M) {
		if let Some(node) = self.nodes.get_mut(&id) {
			node.documentation_mut().add(comment)
		}
	}

	/// Declare the given `id` as a type.
	pub fn declare_type(&mut self, id: Id, cause: M) {
		match self.nodes.get_mut(&id) {
			Some(node) => node.declare_type(cause),
			None => {
				self.nodes.insert(id, Node::new_type(id, cause));
			}
		}
	}

	/// Declare the given `id` as a property.
	pub fn declare_property(&mut self, id: Id, cause: M) {
		match self.nodes.get_mut(&id) {
			Some(node) => node.declare_property(cause),
			None => {
				self.nodes.insert(id, Node::new_property(id, cause));
			}
		}
	}

	/// Declare the given `id` as a layout.
	pub fn declare_layout(&mut self, id: Id, cause: M) {
		match self.nodes.get_mut(&id) {
			Some(node) => node.declare_layout(cause),
			None => {
				self.nodes.insert(id, Node::new_layout(id, cause));
			}
		}
	}

	/// Declare the given `id` as a layout field.
	pub fn declare_layout_field(&mut self, id: Id, cause: M) {
		match self.nodes.get_mut(&id) {
			Some(node) => node.declare_layout_field(cause),
			None => {
				self.nodes.insert(id, Node::new_layout_field(id, cause));
			}
		}
	}

	/// Declare the given `id` as a layout variant.
	pub fn declare_layout_variant(&mut self, id: Id, cause: M) {
		match self.nodes.get_mut(&id) {
			Some(node) => node.declare_layout_variant(cause),
			None => {
				self.nodes.insert(id, Node::new_layout_variant(id, cause));
			}
		}
	}

	/// Declare the given `id` as a list.
	pub fn declare_list(&mut self, id: Id, cause: M) {
		match id {
			Id::Iri(IriIndex::Iri(vocab::Term::Rdf(vocab::Rdf::Nil))) => (),
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
		cause: &M,
	) -> Result<&mut Node<node::Components<M, D>>, Error<M>>
	where
		M: Clone,
	{
		match self.get_mut(id) {
			Some(node) => Ok(node),
			None => Err(Meta(
				error::NodeUnknown {
					id,
					expected_ty: None,
				}
				.into(),
				cause.clone(),
			)),
		}
	}

	#[allow(clippy::type_complexity)]
	pub fn require_type_mut(
		&mut self,
		id: Id,
		cause: &M,
	) -> Result<&mut Meta<ty::Definition<M, D::Type>, M>, Error<M>>
	where
		M: Clone,
	{
		match self.get_mut(id) {
			Some(node) => node.require_type_mut(cause),
			None => Err(Meta(
				error::NodeUnknown {
					id,
					expected_ty: Some(node::Type::Type),
				}
				.into(),
				cause.clone(),
			)),
		}
	}

	pub fn require_property_mut(
		&mut self,
		id: Id,
		cause: &M,
	) -> Result<&mut Meta<prop::Definition<M>, M>, Error<M>>
	where
		M: Clone,
	{
		match self.get_mut(id) {
			Some(node) => node.require_property_mut(cause),
			None => Err(Meta(
				error::NodeUnknown {
					id,
					expected_ty: Some(node::Type::Property),
				}
				.into(),
				cause.clone(),
			)),
		}
	}

	#[allow(clippy::type_complexity)]
	pub fn require_layout(
		&self,
		id: Id,
		cause: &M,
	) -> Result<&Meta<layout::Definition<M, D::Layout>, M>, Error<M>>
	where
		M: Clone,
	{
		match self.get(id) {
			Some(node) => node.require_layout(cause),
			None => Err(Meta(
				error::NodeUnknown {
					id,
					expected_ty: Some(node::Type::Layout),
				}
				.into(),
				cause.clone(),
			)),
		}
	}

	#[allow(clippy::type_complexity)]
	pub fn require_layout_mut(
		&mut self,
		id: Id,
		cause: &M,
	) -> Result<&mut Meta<layout::Definition<M, D::Layout>, M>, Error<M>>
	where
		M: Clone,
	{
		match self.get_mut(id) {
			Some(node) => node.require_layout_mut(cause),
			None => Err(Meta(
				error::NodeUnknown {
					id,
					expected_ty: Some(node::Type::Layout),
				}
				.into(),
				cause.clone(),
			)),
		}
	}

	pub fn require_layout_field(
		&self,
		id: Id,
		cause: &M,
	) -> Result<&Meta<layout::field::Definition<M>, M>, Error<M>>
	where
		M: Clone,
	{
		match self.get(id) {
			Some(node) => node.require_layout_field(cause),
			None => Err(Meta(
				error::NodeUnknown {
					id,
					expected_ty: Some(node::Type::LayoutField),
				}
				.into(),
				cause.clone(),
			)),
		}
	}

	pub fn require_layout_field_mut(
		&mut self,
		id: Id,
		cause: &M,
	) -> Result<&mut Meta<layout::field::Definition<M>, M>, Error<M>>
	where
		M: Clone,
	{
		match self.get_mut(id) {
			Some(node) => node.require_layout_field_mut(cause),
			None => Err(Meta(
				error::NodeUnknown {
					id,
					expected_ty: Some(node::Type::LayoutField),
				}
				.into(),
				cause.clone(),
			)),
		}
	}

	pub fn require_layout_variant(
		&self,
		id: Id,
		cause: &M,
	) -> Result<&Meta<layout::variant::Definition<M>, M>, Error<M>>
	where
		M: Clone,
	{
		match self.get(id) {
			Some(node) => node.require_layout_variant(cause),
			None => Err(Meta(
				error::NodeUnknown {
					id,
					expected_ty: Some(node::Type::LayoutVariant),
				}
				.into(),
				cause.clone(),
			)),
		}
	}

	pub fn require_layout_variant_mut(
		&mut self,
		id: Id,
		cause: &M,
	) -> Result<&mut Meta<layout::variant::Definition<M>, M>, Error<M>>
	where
		M: Clone,
	{
		match self.get_mut(id) {
			Some(node) => node.require_layout_variant_mut(cause),
			None => Err(Meta(
				error::NodeUnknown {
					id,
					expected_ty: Some(node::Type::LayoutVariant),
				}
				.into(),
				cause.clone(),
			)),
		}
	}

	pub fn require_property_or_layout_field_mut(
		&mut self,
		id: Id,
		cause: &M,
	) -> Result<node::PropertyOrLayoutField<M>, Error<M>>
	where
		M: Clone,
	{
		match self.get_mut(id) {
			Some(node) => node.require_property_or_layout_field_mut(cause),
			None => Err(Meta(
				error::NodeUnknown {
					id,
					expected_ty: Some(node::Type::Property),
				}
				.into(),
				cause.clone(),
			)),
		}
	}

	pub fn require_layout_field_or_variant_mut(
		&mut self,
		id: Id,
		cause: &M,
	) -> Result<node::LayoutFieldOrVariant<M>, Error<M>>
	where
		M: Clone,
	{
		match self.get_mut(id) {
			Some(node) => node.require_layout_field_or_variant_mut(cause),
			None => Err(Meta(
				error::NodeUnknown {
					id,
					expected_ty: Some(node::Type::Property),
				}
				.into(),
				cause.clone(),
			)),
		}
	}

	pub fn require_list(&self, id: Id, cause: &M) -> Result<ListRef<M>, Error<M>>
	where
		M: Clone,
	{
		match id {
			Id::Iri(IriIndex::Iri(vocab::Term::Rdf(vocab::Rdf::Nil))) => Ok(ListRef::Nil),
			id => match self.get(id) {
				Some(node) => Ok(ListRef::Cons(node.require_list(cause)?)),
				None => Err(Meta(
					error::NodeUnknown {
						id,
						expected_ty: Some(node::Type::List),
					}
					.into(),
					cause.clone(),
				)),
			},
		}
	}

	pub fn require_list_mut(&mut self, id: Id, cause: &M) -> Result<ListMut<M>, Error<M>>
	where
		M: Clone,
	{
		match id {
			Id::Iri(IriIndex::Iri(vocab::Term::Rdf(vocab::Rdf::Nil))) => Ok(ListMut::Nil),
			id => match self.get_mut(id) {
				Some(node) => Ok(ListMut::Cons(node.require_list_mut(cause)?)),
				None => Err(Meta(
					error::NodeUnknown {
						id,
						expected_ty: Some(node::Type::List),
					}
					.into(),
					cause.clone(),
				)),
			},
		}
	}

	pub fn create_list<
		I: IntoIterator<Item = Meta<vocab::Object<M>, M>>,
		V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>,
	>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		list: I,
	) -> Result<Id, Error<M>>
	where
		M: Clone,
		I::IntoIter: DoubleEndedIterator,
	{
		let mut head = Id::Iri(IriIndex::Iri(vocab::Term::Rdf(vocab::Rdf::Nil)));

		for Meta(item, cause) in list.into_iter().rev() {
			let id = generator.next(vocabulary);

			self.declare_list(id, cause.clone());
			let node = self.get_mut(id).unwrap().as_list_mut().unwrap();
			node.set_first(item, cause.clone())?;
			node.set_rest(head, cause)?;
			head = id;
		}

		Ok(head)
	}

	pub fn create_list_with<I: IntoIterator, C, V>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		list: I,
		mut f: C,
	) -> Result<Id, Error<M>>
	where
		M: Clone,
		I::IntoIter: DoubleEndedIterator,
		V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>,
		C: FnMut(I::Item, &mut Self, &mut V) -> Meta<vocab::Object<M>, M>,
	{
		let mut head = Id::Iri(IriIndex::Iri(vocab::Term::Rdf(vocab::Rdf::Nil)));

		for item in list.into_iter().rev() {
			let id = generator.next(vocabulary);
			let Meta(item, cause) = f(item, self, vocabulary);

			self.declare_list(id, cause.clone());
			let node = self.get_mut(id).unwrap().as_list_mut().unwrap();
			node.set_first(item, cause.clone())?;
			node.set_rest(head, cause)?;
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
		M: Clone,
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

			self.declare_list(id, cause.clone());
			let node = self.get_mut(id).unwrap().as_list_mut().unwrap();
			node.set_first(item, cause.clone())?;
			node.set_rest(head, cause)?;
			head = id;
		}

		Ok(head)
	}
}

impl<M: Clone> Context<M> {
	/// Compute the `use` relation between all the layouts.
	///
	/// A layout is used by another layout if it is the layout of one of its
	/// fields.
	/// The purpose of this function is to declare to each layout how it it used
	/// using the `layout::Definition::add_use` method.
	pub fn compute_uses(&mut self) -> Result<(), Error<M>>
	where
		M: Clone,
	{
		for (id, node) in &self.nodes {
			if let Some(layout) = node.value().layout.as_ref() {
				let sub_layouts = layout.sub_layouts(self)?;

				for sub_layout in &sub_layouts {
					self.layout_relations
						.entry(*sub_layout.layout)
						.or_default()
						.parent
						.push(Meta::new(
							ParentLayout {
								layout: *id,
								connection: sub_layout.connection,
							},
							sub_layout.layout.metadata().clone(),
						))
				}

				self.layout_relations.entry(*id).or_default().sub = sub_layouts
			}
		}

		Ok(())
	}

	pub fn assign_default_layouts<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) where
		M: Merge,
	{
		let mut default_layouts = BTreeMap::new();
		for (id, node) in &self.nodes {
			if let Some(field) = node.as_layout_field() {
				if field.layout().is_none() {
					if let Some(default_layout) = field.default_layout(self) {
						default_layouts.insert(*id, default_layout);
					}
				}
			}
		}

		for (id, default_layout) in default_layouts {
			let default_layout = default_layout.build(self, vocabulary, generator);
			self.get_mut(id)
				.unwrap()
				.as_layout_field_mut()
				.unwrap()
				.replace_layout(default_layout.into());
		}
	}

	/// Assigns default name for layouts/variants that don't have a name yet.
	pub fn assign_default_names(
		&mut self,
		vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
	) -> Result<(), Error<M>>
	where
		M: Clone,
	{
		// Start with the fields.
		let mut default_field_names = BTreeMap::new();
		for (id, node) in &self.nodes {
			if let Some(field) = node.as_layout_field() {
				if let Some(name) = field.default_name(vocabulary, field.metadata().clone()) {
					default_field_names.insert(*id, name);
				}
			}
		}
		for (id, Meta(name, cause)) in default_field_names {
			let field = self.require_layout_field_mut(id, &cause)?;
			if field.name().is_none() {
				field.set_name(name, cause)?;
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
				let parent_layouts = &self.layout_relations.get(id).unwrap().parent;
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
				let parent_layouts = &self.layout_relations.get(id).unwrap().parent;
				if let Some(Meta(name, cause)) = layout.default_name(
					self,
					vocabulary,
					parent_layouts,
					layout.metadata().clone(),
				)? {
					let layout = self.get_mut(*id).unwrap().as_layout_mut().unwrap();
					if layout.name().is_none() {
						layout.set_name(name, cause)?;
					}
				}
			}
		}

		// Now the layouts variants.
		let mut default_variant_names = BTreeMap::new();
		for (id, node) in &self.nodes {
			if let Some(layout) = node.as_layout_variant() {
				if let Some(name) =
					layout.default_name(self, vocabulary, layout.metadata().clone())?
				{
					default_variant_names.insert(*id, name);
				}
			}
		}
		for (id, Meta(name, cause)) in default_variant_names {
			let layout = self.require_layout_variant_mut(id, &cause)?;
			if layout.name().is_none() {
				layout.set_name(name, cause)?;
			}
		}

		Ok(())
	}

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

		self.assign_default_layouts(vocabulary, generator);
		self.compute_uses()?;
		self.assign_default_names(vocabulary)?;

		let mut allocated_shelves = allocated::Shelves::default();
		let mut allocated_nodes = allocated::Nodes::new(&mut allocated_shelves, self.nodes);
		let graph = allocated_shelves.dependency_graph(&allocated_nodes)?;

		let components = graph.strongly_connected_components();

		let ordered_components = components.order_by_depth();

		let mut types_to_build: Vec<_> = allocated_shelves
			.types
			.into_storage()
			.into_iter()
			.map(Option::Some)
			.collect();
		let mut properties_to_build: Vec<_> = allocated_shelves
			.properties
			.into_storage()
			.into_iter()
			.map(Option::Some)
			.collect();
		let mut layouts_to_build: Vec<_> = allocated_shelves
			.layouts
			.into_storage()
			.into_iter()
			.map(Option::Some)
			.collect();

		let mut built_types = Vec::new();
		built_types.resize_with(types_to_build.len(), || None);
		let mut built_properties = Vec::new();
		built_properties.resize_with(properties_to_build.len(), || None);
		let mut built_layouts = Vec::new();
		built_layouts.resize_with(layouts_to_build.len(), || None);

		for i in ordered_components.into_iter().rev() {
			let component = components.get(i).unwrap();
			for item in component {
				let dependencies = crate::Dependencies {
					types: &built_types,
					properties: &built_properties,
					layouts: &built_layouts,
				};

				match item {
					crate::Item::Type(ty_ref) => {
						let (_, Meta(ty, causes)) = types_to_build[ty_ref.index()].take().unwrap();
						let built_ty = ty.build(&mut allocated_nodes, dependencies, causes)?;
						built_types[ty_ref.index()] = Some(built_ty)
					}
					crate::Item::Property(prop_ref) => {
						let (_, Meta(prop, causes)) =
							properties_to_build[prop_ref.index()].take().unwrap();
						let built_prop = prop.build(&mut allocated_nodes, dependencies, causes)?;
						built_properties[prop_ref.index()] = Some(built_prop)
					}
					crate::Item::Layout(layout_ref) => {
						let (_, Meta(layout, causes)) =
							layouts_to_build[layout_ref.index()].take().unwrap();
						let built_layout =
							layout.build(&mut allocated_nodes, dependencies, causes)?;
						built_layouts[layout_ref.index()] = Some(built_layout)
					}
				}
			}
		}

		Ok(Model::from_parts(
			allocated_nodes.into_model_nodes(),
			Shelf::new(built_types.into_iter().map(Option::unwrap).collect()),
			Shelf::new(built_properties.into_iter().map(Option::unwrap).collect()),
			Shelf::new(built_layouts.into_iter().map(Option::unwrap).collect()),
		))
	}
}
