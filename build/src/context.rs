use crate::{
	error, layout, node, prop, ty, Descriptions, Error, ListMut, ListRef, Node, ParentLayout,
	Simplify, SubLayout,
};
use derivative::Derivative;
use locspan::Location;
use shelves::Shelf;
use std::collections::{BTreeMap, HashMap};
use treeldr::{vocab, Caused, Id, Model, Vocabulary, WithCauses};

pub mod allocated;

/// TreeLDR build context.
#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct Context<F, D: Descriptions<F> = crate::StandardDescriptions> {
	/// Nodes.
	nodes: BTreeMap<Id, Node<node::Components<F, D>>>,

	layout_relations: HashMap<Id, LayoutRelations<F>>,

	standard_references: HashMap<Id, Id>,
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
struct LayoutRelations<F> {
	sub: Vec<SubLayout<F>>,
	parent: Vec<WithCauses<ParentLayout, F>>,
}

impl<F, D: Descriptions<F>> Context<F, D> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn define_primitive_datatype(
		&mut self,
		id: Id,
		primitive_type: ty::data::Primitive,
	) -> Result<Id, Error<F>>
	where
		F: Clone + Ord,
	{
		self.declare_type(id, None);
		let ty = self.get_mut(id).unwrap().as_type_mut().unwrap();
		let dt = ty.require_datatype_mut(None)?;
		dt.set_primitive(primitive_type, None)?;
		Ok(id)
	}

	pub fn define_primitive_layout(
		&mut self,
		primitive_layout: layout::Primitive,
	) -> Result<Id, Error<F>>
	where
		F: Clone + Ord,
	{
		let id = Id::Iri(vocab::Term::TreeLdr(vocab::TreeLdr::Primitive(
			primitive_layout,
		)));
		self.declare_layout(id, None);
		let layout = self.get_mut(id).unwrap().as_layout_mut().unwrap();
		layout.set_primitive(primitive_layout.into(), None)?;
		Ok(id)
	}

	pub fn apply_built_in_definitions(
		&mut self,
		vocabulary: &mut Vocabulary,
	) -> Result<(), Error<F>>
	where
		F: Clone + Ord,
	{
		self.define_rdf_types(vocabulary)?;
		self.define_xsd_types()?;
		self.define_treeldr_types()
	}

	pub fn standard_reference(
		&mut self,
		vocabulary: &mut Vocabulary,
		deref_ty: Id,
		cause: Option<Location<F>>,
		deref_cause: Option<Location<F>>,
	) -> Result<Id, Error<F>>
	where
		F: Clone + Ord,
	{
		match self.standard_references.get(&deref_ty).cloned() {
			Some(id) => Ok(id),
			None => {
				let id = self.create_reference(vocabulary, deref_ty, cause, deref_cause)?;
				self.standard_references.insert(deref_ty, id);
				Ok(id)
			}
		}
	}

	pub fn create_reference(
		&mut self,
		vocabulary: &mut Vocabulary,
		target_ty: Id,
		cause: Option<Location<F>>,
		deref_cause: Option<Location<F>>,
	) -> Result<Id, Error<F>>
	where
		F: Clone + Ord,
	{
		let id = Id::Blank(vocabulary.new_blank_label());
		self.create_named_reference(id, target_ty, cause, deref_cause)
	}

	pub fn create_named_reference(
		&mut self,
		id: Id,
		target_ty: Id,
		cause: Option<Location<F>>,
		deref_cause: Option<Location<F>>,
	) -> Result<Id, Error<F>>
	where
		F: Clone + Ord,
	{
		self.declare_layout(id, cause.clone());
		let layout = self.get_mut(id).unwrap().as_layout_mut().unwrap();
		layout.set_type(target_ty, deref_cause)?;
		layout.set_reference(
			Id::Iri(vocab::Term::TreeLdr(vocab::TreeLdr::Primitive(
				treeldr::layout::Primitive::Iri,
			))),
			cause,
		)?;
		Ok(id)
	}

	pub fn define_rdf_types(&mut self, vocabulary: &mut Vocabulary) -> Result<(), Error<F>>
	where
		F: Clone + Ord,
	{
		use vocab::{Rdf, Rdfs, Term};
		// rdfs:Resource
		self.declare_type(Id::Iri(Term::Rdfs(Rdfs::Resource)), None);
		self.declare_layout(Id::Iri(Term::Rdfs(Rdfs::Resource)), None);
		let id_field = Id::Blank(vocabulary.new_blank_label());
		self.declare_layout_field(id_field, None);
		let field_layout =
			self.standard_reference(vocabulary, Id::Iri(Term::Rdfs(Rdfs::Resource)), None, None)?;
		let field = self
			.get_mut(id_field)
			.unwrap()
			.as_layout_field_mut()
			.unwrap();
		field.set_property(Id::Iri(Term::TreeLdr(vocab::TreeLdr::Self_)), None)?;
		field.set_name(treeldr::Name::new("id").unwrap(), None)?;
		field.set_layout(field_layout, None)?;
		let fields_id = self.create_list(vocabulary, [Caused::new(id_field.into_term(), None)])?;
		let layout = self
			.get_mut(Id::Iri(Term::Rdfs(Rdfs::Resource)))
			.unwrap()
			.as_layout_mut()
			.unwrap();
		layout.set_fields(fields_id, None)?;

		// rdfs:Class
		self.declare_type(Id::Iri(Term::Rdfs(Rdfs::Class)), None);

		// rdf:Property
		self.declare_type(Id::Iri(Term::Rdf(Rdf::Property)), None);

		// rdf:type
		self.declare_property(Id::Iri(Term::Rdf(Rdf::Type)), None);
		let prop = self
			.get_mut(Id::Iri(Term::Rdf(Rdf::Type)))
			.unwrap()
			.as_property_mut()
			.unwrap();
		prop.set_domain(Id::Iri(Term::Rdfs(Rdfs::Resource)), None);
		prop.set_range(Id::Iri(Term::Rdfs(Rdfs::Class)), None)?;

		// rdf:List
		self.declare_type(Id::Iri(Term::Rdf(Rdf::List)), None);
		let list = self
			.get_mut(Id::Iri(Term::Rdf(Rdf::List)))
			.unwrap()
			.as_type_mut()
			.unwrap();
		list.declare_property(Id::Iri(Term::Rdf(Rdf::First)), None)?;
		list.declare_property(Id::Iri(Term::Rdf(Rdf::Rest)), None)?;

		// rdf:first
		self.declare_property(Id::Iri(Term::Rdf(Rdf::First)), None);
		let prop = self
			.get_mut(Id::Iri(Term::Rdf(Rdf::First)))
			.unwrap()
			.as_property_mut()
			.unwrap();
		prop.set_domain(Id::Iri(Term::Rdf(Rdf::List)), None);
		prop.set_range(Id::Iri(Term::Rdfs(Rdfs::Resource)), None)?;

		// rdf:rest
		self.declare_property(Id::Iri(Term::Rdf(Rdf::Rest)), None);
		let prop = self
			.get_mut(Id::Iri(Term::Rdf(Rdf::Rest)))
			.unwrap()
			.as_property_mut()
			.unwrap();
		prop.set_domain(Id::Iri(Term::Rdf(Rdf::List)), None);
		prop.set_range(Id::Iri(Term::Rdf(Rdf::List)), None)
	}

	pub fn define_xsd_types(&mut self) -> Result<(), Error<F>>
	where
		F: Clone + Ord,
	{
		use vocab::{Term, Xsd};
		self.define_primitive_datatype(
			Id::Iri(Term::Xsd(Xsd::String)),
			ty::data::Primitive::String,
		)?;
		Ok(())
	}

	pub fn define_treeldr_types(&mut self) -> Result<(), Error<F>>
	where
		F: Clone + Ord,
	{
		use layout::Primitive;

		self.declare_property(Id::Iri(vocab::Term::TreeLdr(vocab::TreeLdr::Self_)), None);
		let prop = self
			.get_mut(Id::Iri(vocab::Term::TreeLdr(vocab::TreeLdr::Self_)))
			.unwrap()
			.as_property_mut()
			.unwrap();
		prop.set_range(Id::Iri(vocab::Term::Rdfs(vocab::Rdfs::Resource)), None)?;

		self.define_primitive_layout(Primitive::Boolean)?;
		self.define_primitive_layout(Primitive::Integer)?;
		self.define_primitive_layout(Primitive::UnsignedInteger)?;
		self.define_primitive_layout(Primitive::Float)?;
		self.define_primitive_layout(Primitive::Double)?;
		self.define_primitive_layout(Primitive::String)?;
		self.define_primitive_layout(Primitive::Time)?;
		self.define_primitive_layout(Primitive::Date)?;
		self.define_primitive_layout(Primitive::DateTime)?;
		self.define_primitive_layout(Primitive::Iri)?;
		self.define_primitive_layout(Primitive::Uri)?;
		self.define_primitive_layout(Primitive::Url)?;

		Ok(())
	}

	pub fn try_map<G: Descriptions<F>, E>(
		&self,
		map: &impl crate::TryMap<F, E, D, G>,
		vocabulary: &mut Vocabulary,
	) -> Result<Context<F, G>, E>
	where
		F: Clone,
	{
		let mut target = Context::new();

		for (id, node) in &self.nodes {
			let mapped_node = node
				.clone()
				.try_map(|desc| desc.try_map(map, self, &mut target, vocabulary))?;
			target.nodes.insert(*id, mapped_node);
		}

		Ok(target)
	}

	pub fn simplify(
		&self,
		vocabulary: &mut Vocabulary,
	) -> Result<Context<F>, <D as Simplify<F>>::Error>
	where
		D: Simplify<F>,
		F: Clone + Ord,
	{
		let map = D::TryMap::default();
		self.try_map(&map, vocabulary)
	}

	/// Returns the node associated to the given `Id`, if any.
	pub fn get(&self, id: Id) -> Option<&Node<node::Components<F, D>>> {
		self.nodes.get(&id)
	}

	/// Returns a mutable reference to the node associated to the given `Id`, if any.
	pub fn get_mut(&mut self, id: Id) -> Option<&mut Node<node::Components<F, D>>> {
		self.nodes.get_mut(&id)
	}

	pub fn nodes(&self) -> impl Iterator<Item = (Id, &Node<node::Components<F, D>>)> {
		self.nodes.iter().map(|(id, node)| (*id, node))
	}

	pub fn nodes_mut(&mut self) -> impl Iterator<Item = (Id, &mut Node<node::Components<F, D>>)> {
		self.nodes.iter_mut().map(|(id, node)| (*id, node))
	}

	/// Inserts the given node to the context.
	///
	/// Replaces any previous node with the same [`Node::id`].
	pub fn insert(
		&mut self,
		node: Node<node::Components<F, D>>,
	) -> Option<Node<node::Components<F, D>>> {
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

	/// Declare the given `id` as a layout field.
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

	/// Declare the given `id` as a layout variant.
	pub fn declare_layout_variant(&mut self, id: Id, cause: Option<Location<F>>)
	where
		F: Ord,
	{
		match self.nodes.get_mut(&id) {
			Some(node) => node.declare_layout_variant(cause),
			None => {
				self.nodes.insert(id, Node::new_layout_variant(id, cause));
			}
		}
	}

	/// Declare the given `id` as a list.
	pub fn declare_list(&mut self, id: Id, cause: Option<Location<F>>)
	where
		F: Ord,
	{
		match id {
			Id::Iri(vocab::Term::Rdf(vocab::Rdf::Nil)) => (),
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
	) -> Result<&mut Node<node::Components<F, D>>, Error<F>>
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

	#[allow(clippy::type_complexity)]
	pub fn require_type_mut(
		&mut self,
		id: Id,
		cause: Option<Location<F>>,
	) -> Result<&mut WithCauses<ty::Definition<F, D::Type>, F>, Error<F>>
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

	#[allow(clippy::type_complexity)]
	pub fn require_layout(
		&self,
		id: Id,
		cause: Option<Location<F>>,
	) -> Result<&WithCauses<layout::Definition<F, D::Layout>, F>, Error<F>>
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

	#[allow(clippy::type_complexity)]
	pub fn require_layout_mut(
		&mut self,
		id: Id,
		cause: Option<Location<F>>,
	) -> Result<&mut WithCauses<layout::Definition<F, D::Layout>, F>, Error<F>>
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

	pub fn require_layout_field(
		&self,
		id: Id,
		cause: Option<Location<F>>,
	) -> Result<&WithCauses<layout::field::Definition<F>, F>, Error<F>>
	where
		F: Clone,
	{
		match self.get(id) {
			Some(node) => node.require_layout_field(cause),
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

	pub fn require_layout_variant(
		&self,
		id: Id,
		cause: Option<Location<F>>,
	) -> Result<&WithCauses<layout::variant::Definition<F>, F>, Error<F>>
	where
		F: Clone,
	{
		match self.get(id) {
			Some(node) => node.require_layout_variant(cause),
			None => Err(Caused::new(
				error::NodeUnknown {
					id,
					expected_ty: Some(node::Type::LayoutVariant),
				}
				.into(),
				cause,
			)),
		}
	}

	pub fn require_layout_variant_mut(
		&mut self,
		id: Id,
		cause: Option<Location<F>>,
	) -> Result<&mut WithCauses<layout::variant::Definition<F>, F>, Error<F>>
	where
		F: Clone,
	{
		match self.get_mut(id) {
			Some(node) => node.require_layout_variant_mut(cause),
			None => Err(Caused::new(
				error::NodeUnknown {
					id,
					expected_ty: Some(node::Type::LayoutVariant),
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

	pub fn require_layout_field_or_variant_mut(
		&mut self,
		id: Id,
		cause: Option<Location<F>>,
	) -> Result<node::LayoutFieldOrVariant<F>, Error<F>>
	where
		F: Clone,
	{
		match self.get_mut(id) {
			Some(node) => node.require_layout_field_or_variant_mut(cause),
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

	pub fn require_list(&self, id: Id, cause: Option<Location<F>>) -> Result<ListRef<F>, Error<F>>
	where
		F: Clone,
	{
		match id {
			Id::Iri(vocab::Term::Rdf(vocab::Rdf::Nil)) => Ok(ListRef::Nil),
			id => match self.get(id) {
				Some(node) => Ok(ListRef::Cons(node.require_list(cause)?)),
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

	pub fn require_list_mut(
		&mut self,
		id: Id,
		cause: Option<Location<F>>,
	) -> Result<ListMut<F>, Error<F>>
	where
		F: Clone,
	{
		match id {
			Id::Iri(vocab::Term::Rdf(vocab::Rdf::Nil)) => Ok(ListMut::Nil),
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

	pub fn create_list<I: IntoIterator<Item = Caused<vocab::Object<F>, F>>>(
		&mut self,
		vocabulary: &mut Vocabulary,
		list: I,
	) -> Result<Id, Error<F>>
	where
		F: Clone + Ord,
		I::IntoIter: DoubleEndedIterator,
	{
		let mut head = Id::Iri(vocab::Term::Rdf(vocab::Rdf::Nil));

		for item in list.into_iter().rev() {
			let id = Id::Blank(vocabulary.new_blank_label());
			let (item, cause) = item.into_parts();

			self.declare_list(id, cause.clone());
			let node = self.get_mut(id).unwrap().as_list_mut().unwrap();
			node.set_first(item, cause.clone())?;
			node.set_rest(head, cause)?;
			head = id;
		}

		Ok(head)
	}

	pub fn create_list_with<I: IntoIterator, C>(
		&mut self,
		vocabulary: &mut Vocabulary,
		list: I,
		mut f: C,
	) -> Result<Id, Error<F>>
	where
		F: Clone + Ord,
		I::IntoIter: DoubleEndedIterator,
		C: FnMut(I::Item, &mut Self, &mut Vocabulary) -> Caused<vocab::Object<F>, F>,
	{
		let mut head = Id::Iri(vocab::Term::Rdf(vocab::Rdf::Nil));

		for item in list.into_iter().rev() {
			let id = Id::Blank(vocabulary.new_blank_label());
			let (item, cause) = f(item, self, vocabulary).into_parts();

			self.declare_list(id, cause.clone());
			let node = self.get_mut(id).unwrap().as_list_mut().unwrap();
			node.set_first(item, cause.clone())?;
			node.set_rest(head, cause)?;
			head = id;
		}

		Ok(head)
	}

	pub fn try_create_list_with<E, I: IntoIterator, C>(
		&mut self,
		vocabulary: &mut Vocabulary,
		list: I,
		mut f: C,
	) -> Result<Id, E>
	where
		F: Clone + Ord,
		E: From<Error<F>>,
		I::IntoIter: DoubleEndedIterator,
		C: FnMut(I::Item, &mut Self, &mut Vocabulary) -> Result<Caused<vocab::Object<F>, F>, E>,
	{
		let mut head = Id::Iri(vocab::Term::Rdf(vocab::Rdf::Nil));

		for item in list.into_iter().rev() {
			let id = Id::Blank(vocabulary.new_blank_label());
			let (item, cause) = f(item, self, vocabulary)?.into_parts();

			self.declare_list(id, cause.clone());
			let node = self.get_mut(id).unwrap().as_list_mut().unwrap();
			node.set_first(item, cause.clone())?;
			node.set_rest(head, cause)?;
			head = id;
		}

		Ok(head)
	}
}

impl<F: Clone + Ord> Context<F> {
	/// Compute the `use` relation between all the layouts.
	///
	/// A layout is used by another layout if it is the layout of one of its
	/// fields.
	/// The purpose of this function is to declare to each layout how it it used
	/// using the `layout::Definition::add_use` method.
	pub fn compute_uses(&mut self) -> Result<(), Error<F>>
	where
		F: Ord + Clone,
	{
		for (id, node) in &self.nodes {
			if let Some(layout) = node.value().layout.with_causes() {
				let sub_layouts = layout.sub_layouts(self)?;

				for sub_layout in &sub_layouts {
					self.layout_relations
						.entry(*sub_layout.layout)
						.or_default()
						.parent
						.push(WithCauses::new(
							ParentLayout {
								layout: *id,
								connection: sub_layout.connection,
							},
							sub_layout.layout.causes().clone(),
						))
				}

				self.layout_relations.entry(*id).or_default().sub = sub_layouts
			}
		}

		Ok(())
	}

	pub fn assign_default_layouts(&mut self, vocabulary: &mut Vocabulary) {
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
			let default_layout = default_layout.build(self, vocabulary);
			self.get_mut(id)
				.unwrap()
				.as_layout_field_mut()
				.unwrap()
				.replace_layout(default_layout.into());
		}
	}

	/// Assigns default name for layouts/variants that don't have a name yet.
	pub fn assign_default_names(&mut self, vocabulary: &Vocabulary) -> Result<(), Error<F>>
	where
		F: Ord + Clone,
	{
		// Start with the fields.
		let mut default_field_names = BTreeMap::new();
		for (id, node) in &self.nodes {
			if let Some(field) = node.as_layout_field() {
				if let Some(name) =
					field.default_name(vocabulary, field.causes().preferred().cloned())
				{
					default_field_names.insert(*id, name);
				}
			}
		}
		for (id, name) in default_field_names {
			let (name, cause) = name.into_parts();
			let field = self.require_layout_field_mut(id, cause.clone())?;
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
				if let Some(name) = layout.default_name(
					self,
					vocabulary,
					parent_layouts,
					layout.causes().preferred().cloned(),
				)? {
					let (name, cause) = name.into_parts();
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
					layout.default_name(self, vocabulary, layout.causes().preferred().cloned())?
				{
					default_variant_names.insert(*id, name);
				}
			}
		}
		for (id, name) in default_variant_names {
			let (name, cause) = name.into_parts();
			let layout = self.require_layout_variant_mut(id, cause.clone())?;
			if layout.name().is_none() {
				layout.set_name(name, cause)?;
			}
		}

		Ok(())
	}

	pub fn build(mut self, vocabulary: &mut Vocabulary) -> Result<Model<F>, Error<F>>
	where
		F: Ord + Clone,
	{
		use crate::utils::SccGraph;
		use crate::Build;

		self.assign_default_layouts(vocabulary);
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
						let (_, ty) = types_to_build[ty_ref.index()].take().unwrap();
						let (ty, causes) = ty.into_parts();
						let built_ty = ty.build(&mut allocated_nodes, dependencies, causes)?;
						built_types[ty_ref.index()] = Some(built_ty)
					}
					crate::Item::Property(prop_ref) => {
						let (_, prop) = properties_to_build[prop_ref.index()].take().unwrap();
						let (prop, causes) = prop.into_parts();
						let built_prop = prop.build(&mut allocated_nodes, dependencies, causes)?;
						built_properties[prop_ref.index()] = Some(built_prop)
					}
					crate::Item::Layout(layout_ref) => {
						let (_, layout) = layouts_to_build[layout_ref.index()].take().unwrap();
						let (layout, causes) = layout.into_parts();
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
