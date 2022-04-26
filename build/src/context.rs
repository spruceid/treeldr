use crate::{
	error, layout, list, node, prop, ty, Descriptions, Error, ListMut, ListRef, Node, ParentLayout,
	SubLayout,
};
use derivative::Derivative;
use locspan::Location;
use shelves::{Ref, Shelf};
use std::collections::{BTreeMap, HashMap};
use treeldr::{vocab, Caused, Id, MaybeSet, Model, Vocabulary, WithCauses};

/// TreeLDR build context.
pub struct Context<'v, F, D: Descriptions<F>> {
	/// Vocabulary.
	vocab: &'v mut Vocabulary,

	/// Nodes.
	nodes: BTreeMap<Id, Node<node::Components<F, D>>>,

	layout_relations: HashMap<Id, LayoutRelations<F>>,
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
struct LayoutRelations<F> {
	sub: Vec<SubLayout<F>>,
	parent: Vec<WithCauses<ParentLayout, F>>,
}

impl<'v, F, D: Descriptions<F>> Context<'v, F, D> {
	pub fn new(vocab: &'v mut Vocabulary) -> Self {
		Self {
			vocab,
			nodes: BTreeMap::new(),
			layout_relations: HashMap::new(),
		}
	}

	pub fn define_native_type(
		&mut self,
		id: Id,
		native_layout: layout::Native,
		cause: Option<Location<F>>,
	) -> Result<Id, Error<F>>
	where
		F: Clone + Ord,
	{
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
			Id::Iri(vocab::Term::Xsd(vocab::Xsd::Boolean)),
			layout::Native::Boolean,
			None,
		)?;
		self.define_native_type(
			Id::Iri(vocab::Term::Xsd(vocab::Xsd::Int)),
			layout::Native::Integer,
			None,
		)?;
		self.define_native_type(
			Id::Iri(vocab::Term::Xsd(vocab::Xsd::Integer)),
			layout::Native::Integer,
			None,
		)?;
		self.define_native_type(
			Id::Iri(vocab::Term::Xsd(vocab::Xsd::PositiveInteger)),
			layout::Native::PositiveInteger,
			None,
		)?;
		self.define_native_type(
			Id::Iri(vocab::Term::Xsd(vocab::Xsd::Float)),
			layout::Native::Float,
			None,
		)?;
		self.define_native_type(
			Id::Iri(vocab::Term::Xsd(vocab::Xsd::Double)),
			layout::Native::Double,
			None,
		)?;
		self.define_native_type(
			Id::Iri(vocab::Term::Xsd(vocab::Xsd::String)),
			layout::Native::String,
			None,
		)?;
		self.define_native_type(
			Id::Iri(vocab::Term::Xsd(vocab::Xsd::Time)),
			layout::Native::Time,
			None,
		)?;
		self.define_native_type(
			Id::Iri(vocab::Term::Xsd(vocab::Xsd::Date)),
			layout::Native::Date,
			None,
		)?;
		self.define_native_type(
			Id::Iri(vocab::Term::Xsd(vocab::Xsd::DateTime)),
			layout::Native::DateTime,
			None,
		)?;
		self.define_native_type(
			Id::Iri(vocab::Term::Xsd(vocab::Xsd::AnyUri)),
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
		use layout::PseudoDescription;
		let mut deref_map = BTreeMap::new();

		for (id, node) in &self.nodes {
			if let Some(layout) = node.as_layout() {
				if let Some(desc) = layout.description() {
					if let Some(layout::Description::Reference(target_layout_id)) =
						desc.inner().as_standard()
					{
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
		let mut depth_map: BTreeMap<_, _> = deref_map.keys().map(|id| (*id, 0)).collect();
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

	/// Compute the `use` relation between all the layouts.
	///
	/// A layout is used by another layout if it is the layout of one of its
	/// fields.
	/// The purpose of this function is to declare to each layout how it it used
	/// using the `layout::Definition::add_use` method.
	pub fn compute_uses(&mut self) -> Result<(), D::Error>
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

	/// Assigns default name for layouts/variants that don't have a name yet.
	pub fn assign_default_names(&mut self) -> Result<(), Error<F>>
	where
		F: Ord + Clone,
	{
		// Start with the fields.
		let mut default_field_names = BTreeMap::new();
		for (id, node) in &self.nodes {
			if let Some(field) = node.as_layout_field() {
				if let Some(name) = field.default_name(self, field.causes().preferred().cloned()) {
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
					layout.default_name(self, layout.causes().preferred().cloned())?
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

	pub fn build(mut self) -> Result<Model<F>, D::Error>
	where
		F: Ord + Clone,
	{
		use crate::Build;
		self.resolve_references()?;
		self.compute_uses()?;
		self.assign_default_names()?;

		let mut allocated_shelves = AllocatedShelves::default();
		let allocated_nodes = AllocatedNodes::new(&mut allocated_shelves, self.nodes);

		use treeldr::utils::SccGraph;

		struct Graph<F> {
			items: Vec<crate::Item<F>>,
			ty_dependencies: Vec<Vec<crate::Item<F>>>,
			prop_dependencies: Vec<Vec<crate::Item<F>>>,
			layout_dependencies: Vec<Vec<crate::Item<F>>>,
		}

		impl<F> SccGraph for Graph<F> {
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

		let mut graph: Graph<F> = Graph {
			items: Vec::with_capacity(
				allocated_shelves.layouts.len()
					+ allocated_shelves.properties.len()
					+ allocated_shelves.types.len(),
			),
			ty_dependencies: Vec::with_capacity(allocated_shelves.types.len()),
			prop_dependencies: Vec::with_capacity(allocated_shelves.properties.len()),
			layout_dependencies: Vec::with_capacity(allocated_shelves.layouts.len()),
		};

		for (ty_ref, (_, ty)) in &allocated_shelves.types {
			graph.items.push(crate::Item::Type(ty_ref.cast()));
			let dependencies = ty.dependencies(&allocated_nodes, ty.causes())?;
			graph.ty_dependencies.push(dependencies)
		}

		for (prop_ref, (_, prop)) in &allocated_shelves.properties {
			graph.items.push(crate::Item::Property(prop_ref.cast()));
			let dependencies = prop.dependencies(&allocated_nodes, prop.causes())?;
			graph.prop_dependencies.push(dependencies)
		}

		for (layout_ref, (_, layout)) in &allocated_shelves.layouts {
			graph.items.push(crate::Item::Layout(layout_ref.cast()));
			let dependencies = layout.dependencies(&allocated_nodes, layout.causes())?;
			graph.layout_dependencies.push(dependencies)
		}

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
						let built_ty = ty.build(&allocated_nodes, dependencies, causes)?;
						built_types[ty_ref.index()] = Some(built_ty)
					}
					crate::Item::Property(prop_ref) => {
						let (_, prop) = properties_to_build[prop_ref.index()].take().unwrap();
						let (prop, causes) = prop.into_parts();
						let built_prop = prop.build(&allocated_nodes, dependencies, causes)?;
						built_properties[prop_ref.index()] = Some(built_prop)
					}
					crate::Item::Layout(layout_ref) => {
						let (_, layout) = layouts_to_build[layout_ref.index()].take().unwrap();
						let (layout, causes) = layout.into_parts();
						let built_layout = layout.build(&allocated_nodes, dependencies, causes)?;
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

	/// Returns a reference to the vocabulary.
	pub fn vocabulary(&self) -> &Vocabulary {
		self.vocab
	}

	/// Returns a mutable reference to the vocabulary.
	pub fn vocabulary_mut(&mut self) -> &mut Vocabulary {
		self.vocab
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
		list: I,
	) -> Result<Id, Error<F>>
	where
		F: Clone + Ord,
		I::IntoIter: DoubleEndedIterator,
	{
		let mut head = Id::Iri(vocab::Term::Rdf(vocab::Rdf::Nil));

		for item in list.into_iter().rev() {
			let id = Id::Blank(self.vocab.new_blank_label());
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
		list: I,
		mut f: C,
	) -> Result<Id, Error<F>>
	where
		F: Clone + Ord,
		I::IntoIter: DoubleEndedIterator,
		C: FnMut(I::Item, &mut Self) -> Caused<vocab::Object<F>, F>,
	{
		let mut head = Id::Iri(vocab::Term::Rdf(vocab::Rdf::Nil));

		for item in list.into_iter().rev() {
			let id = Id::Blank(self.vocab.new_blank_label());
			let (item, cause) = f(item, self).into_parts();

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
		list: I,
		mut f: C,
	) -> Result<Id, E>
	where
		F: Clone + Ord,
		E: From<Error<F>>,
		I::IntoIter: DoubleEndedIterator,
		C: FnMut(I::Item, &mut Self) -> Result<Caused<vocab::Object<F>, F>, E>,
	{
		let mut head = Id::Iri(vocab::Term::Rdf(vocab::Rdf::Nil));

		for item in list.into_iter().rev() {
			let id = Id::Blank(self.vocab.new_blank_label());
			let (item, cause) = f(item, self)?.into_parts();

			self.declare_list(id, cause.clone());
			let node = self.get_mut(id).unwrap().as_list_mut().unwrap();
			node.set_first(item, cause.clone())?;
			node.set_rest(head, cause)?;
			head = id;
		}

		Ok(head)
	}
}

pub struct AllocatedComponents<F> {
	ty: MaybeSet<Ref<treeldr::ty::Definition<F>>, F>,
	property: MaybeSet<Ref<treeldr::prop::Definition<F>>, F>,
	layout: MaybeSet<Ref<treeldr::layout::Definition<F>>, F>,
	layout_field: MaybeSet<layout::field::Definition<F>, F>,
	layout_variant: MaybeSet<layout::variant::Definition<F>, F>,
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

impl<F> From<Node<AllocatedComponents<F>>> for treeldr::Node<F> {
	fn from(n: Node<AllocatedComponents<F>>) -> treeldr::Node<F> {
		let (id, label, doc, value) = n.into_parts();

		treeldr::Node::from_parts(id, label, value.ty, value.property, value.layout, doc)
	}
}

#[allow(clippy::type_complexity)]
pub struct AllocatedShelves<F, D: Descriptions<F>> {
	types: Shelf<Vec<(Id, WithCauses<ty::Definition<F, D::Type>, F>)>>,
	properties: Shelf<Vec<(Id, WithCauses<prop::Definition<F>, F>)>>,
	layouts: Shelf<Vec<(Id, WithCauses<layout::Definition<F, D::Layout>, F>)>>,
}

impl<F, D: Descriptions<F>> Default for AllocatedShelves<F, D> {
	fn default() -> Self {
		Self {
			types: Shelf::default(),
			properties: Shelf::default(),
			layouts: Shelf::default(),
		}
	}
}

pub struct AllocatedNodes<F> {
	nodes: BTreeMap<Id, Node<AllocatedComponents<F>>>,
}

impl<F: Clone> AllocatedNodes<F> {
	pub fn new<D: Descriptions<F>>(
		shelves: &mut AllocatedShelves<F, D>,
		nodes: BTreeMap<Id, Node<node::Components<F, D>>>,
	) -> Self {
		// Step 1: allocate each type/property/layout.
		let mut allocated_nodes = BTreeMap::new();
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
