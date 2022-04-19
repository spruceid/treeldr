use crate::{
	error, layout, list, node, Definitions, Error, ListMut, ListRef, Node, ParentLayout, SubLayout,
};
use derivative::Derivative;
use locspan::Location;
use shelves::{Ref, Shelf};
use std::collections::HashMap;
use treeldr::{vocab, Caused, Id, MaybeSet, Model, Vocabulary, WithCauses};

/// TreeLDR build context.
#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct Context<F, D: Definitions<F>> {
	/// Vocabulary.
	vocab: Vocabulary,

	/// Nodes.
	nodes: HashMap<Id, Node<node::Components<F, D>>>,

	layout_relations: HashMap<Id, LayoutRelations<F>>,
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
struct LayoutRelations<F> {
	sub: Vec<SubLayout<F>>,
	parent: Vec<WithCauses<ParentLayout, F>>,
}

impl<F, D: Definitions<F>> Context<F, D> {
	/// Creates a new empty context.
	pub fn new() -> Self {
		Self::default()
	}

	pub fn with_vocabulary(vocab: Vocabulary) -> Self {
		Self {
			vocab,
			nodes: HashMap::new(),
			layout_relations: HashMap::new(),
		}
	}

	pub fn into_vocabulary(self) -> Vocabulary {
		self.vocab
	}

	// pub fn define_native_type(
	// 	&mut self,
	// 	iri: IriBuf,
	// 	native_layout: layout::Native,
	// 	cause: Option<Location<F>>,
	// ) -> Result<Id, Error<F>>
	// where
	// 	F: Clone + Ord,
	// 	D::Type: From<ty::Definition<F>>,
	// 	D::Layout: From<layout::Definition<F>>
	// {
	// 	let id = Id::Iri(vocab::Term::from_iri(iri, self.vocabulary_mut()));
	// 	self.declare_type(id, cause.clone(), |_| ty::Definition::new().into());
	// 	self.declare_layout(id, cause.clone(), |id| layout::Definition::new(id).into());
	// 	let layout = self.get_mut(id).unwrap().as_layout_mut().unwrap();
	// 	layout.set_native(native_layout, cause.clone())?;
	// 	layout.set_type(id, cause)?;
	// 	Ok(id)
	// }

	// pub fn define_xml_types(&mut self) -> Result<(), Error<F>>
	// where
	// 	F: Clone + Ord,
	// {
	// 	self.define_native_type(
	// 		IriBuf::new("http://www.w3.org/2001/XMLSchema#boolean").unwrap(),
	// 		layout::Native::Boolean,
	// 		None,
	// 	)?;
	// 	self.define_native_type(
	// 		IriBuf::new("http://www.w3.org/2001/XMLSchema#int").unwrap(),
	// 		layout::Native::Integer,
	// 		None,
	// 	)?;
	// 	self.define_native_type(
	// 		IriBuf::new("http://www.w3.org/2001/XMLSchema#integer").unwrap(),
	// 		layout::Native::Integer,
	// 		None,
	// 	)?;
	// 	self.define_native_type(
	// 		IriBuf::new("http://www.w3.org/2001/XMLSchema#positiveInteger").unwrap(),
	// 		layout::Native::PositiveInteger,
	// 		None,
	// 	)?;
	// 	self.define_native_type(
	// 		IriBuf::new("http://www.w3.org/2001/XMLSchema#float").unwrap(),
	// 		layout::Native::Float,
	// 		None,
	// 	)?;
	// 	self.define_native_type(
	// 		IriBuf::new("http://www.w3.org/2001/XMLSchema#double").unwrap(),
	// 		layout::Native::Double,
	// 		None,
	// 	)?;
	// 	self.define_native_type(
	// 		IriBuf::new("http://www.w3.org/2001/XMLSchema#string").unwrap(),
	// 		layout::Native::String,
	// 		None,
	// 	)?;
	// 	self.define_native_type(
	// 		IriBuf::new("http://www.w3.org/2001/XMLSchema#time").unwrap(),
	// 		layout::Native::Time,
	// 		None,
	// 	)?;
	// 	self.define_native_type(
	// 		IriBuf::new("http://www.w3.org/2001/XMLSchema#date").unwrap(),
	// 		layout::Native::Date,
	// 		None,
	// 	)?;
	// 	self.define_native_type(
	// 		IriBuf::new("http://www.w3.org/2001/XMLSchema#dateTime").unwrap(),
	// 		layout::Native::DateTime,
	// 		None,
	// 	)?;
	// 	self.define_native_type(
	// 		IriBuf::new("http://www.w3.org/2001/XMLSchema#anyURI").unwrap(),
	// 		layout::Native::Uri,
	// 		None,
	// 	)?;

	// 	Ok(())
	// }

	// /// Resolve all the reference layouts.
	// ///
	// /// Checks that the type of a reference layout (`&T`) is equal to the type of the target layout (`T`).
	// /// If no type is defined for the reference layout, it is set to the correct type.
	// pub fn resolve_references(&mut self) -> Result<(), Error<F>>
	// where
	// 	F: Ord + Clone,
	// {
	// 	let mut deref_map = HashMap::new();

	// 	for (id, node) in &self.nodes {
	// 		if let Some(layout) = node.as_layout() {
	// 			if let Some(desc) = layout.description() {
	// 				if let layout::Description::Reference(target_layout_id) = desc.inner() {
	// 					deref_map.insert(
	// 						*id,
	// 						Caused::new(*target_layout_id, desc.causes().preferred().cloned()),
	// 					);
	// 				}
	// 			}
	// 		}
	// 	}

	// 	// Assign a depth to each reference.
	// 	// The depth correspond the the reference nesting level (`&&&&T` => 4 nesting level => depth 3).
	// 	// References with higher depth must be resolved first.
	// 	let mut depth_map: HashMap<_, _> = deref_map.keys().map(|id| (*id, 0)).collect();
	// 	let mut stack: Vec<_> = deref_map.keys().map(|id| (*id, 0)).collect();
	// 	while let Some((id, depth)) = stack.pop() {
	// 		let current_depth = depth_map[&id];
	// 		if depth > current_depth {
	// 			if current_depth > 0 {
	// 				panic!("cycling reference")
	// 			}

	// 			depth_map.insert(id, depth);
	// 			if let Some(target_layout_id) = deref_map.get(&id) {
	// 				stack.push((*target_layout_id.inner(), depth + 1));
	// 			}
	// 		}
	// 	}

	// 	// Sort references by depth (highest first).
	// 	let mut by_depth: Vec<_> = deref_map.into_iter().collect();
	// 	by_depth.sort_by(|(a, _), (b, _)| depth_map[b].cmp(&depth_map[a]));

	// 	// Actually resolve the references.
	// 	for (id, target_layout_id) in by_depth {
	// 		let (target_layout_id, cause) = target_layout_id.into_parts();
	// 		let target_layout = self.require_layout(target_layout_id, cause.clone())?;
	// 		let (target_ty_id, ty_cause) = target_layout.require_ty(cause)?.clone().into_parts();
	// 		self.get_mut(id)
	// 			.unwrap()
	// 			.as_layout_mut()
	// 			.unwrap()
	// 			.set_type(target_ty_id, ty_cause.into_preferred())?
	// 	}

	// 	Ok(())
	// }

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
		use crate::Layout;

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
		use crate::Layout;

		// Start with the layouts.
		let mut default_layout_names = HashMap::new();
		for (id, node) in &self.nodes {
			if let Some(layout) = node.as_layout() {
				let parent_layouts = &self.layout_relations.get(id).unwrap().parent;
				if let Some(name) = layout.default_name(
					self,
					parent_layouts,
					layout.causes().preferred().cloned(),
				)? {
					default_layout_names.insert(*id, name);
				}
			}
		}
		for (id, name) in default_layout_names {
			let (name, cause) = name.into_parts();
			let layout = self.require_layout_mut(id, cause.clone())?;
			if layout.name().is_none() {
				layout.set_name(name, cause)?;
			}
		}

		// Now the layouts variants.
		let mut default_variant_names = HashMap::new();
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

	pub fn build(mut self) -> Result<Model<F>, (D::Error, Vocabulary)>
	where
		F: Ord + Clone,
	{
		use crate::Build;
		// if let Err(e) = self.resolve_references() {
		// 	return Err((e, self.into_vocabulary()));
		// }

		if let Err(e) = self.compute_uses() {
			return Err((e.into(), self.into_vocabulary()));
		}

		if let Err(e) = self.assign_default_names() {
			return Err((e.into(), self.into_vocabulary()));
		}

		let mut allocated_shelves = AllocatedShelves::default();
		let allocated_nodes = AllocatedNodes::new(&mut allocated_shelves, self.nodes);

		use treeldr::utils::SccGraph;

		struct Graph<F> {
			items: Vec<crate::Item<F>>,
			ty_dependencies: Vec<Vec<crate::Item<F>>>,
			prop_dependencies: Vec<Vec<crate::Item<F>>>,
			layout_dependencies: Vec<Vec<crate::Item<F>>>,
		}

		impl<F> treeldr::utils::SccGraph for Graph<F> {
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

		for (ty_ref, (id, ty)) in &allocated_shelves.types {
			graph.items.push(crate::Item::Type(ty_ref.cast()));
			match ty.dependencies(*id, &allocated_nodes, ty.causes()) {
				Ok(dependencies) => graph.ty_dependencies.push(dependencies),
				Err(e) => return Err((e.into(), self.vocab)),
			}
		}

		for (prop_ref, (id, prop)) in &allocated_shelves.properties {
			graph.items.push(crate::Item::Property(prop_ref.cast()));
			match prop.dependencies(*id, &allocated_nodes, prop.causes()) {
				Ok(dependencies) => graph.prop_dependencies.push(dependencies),
				Err(e) => return Err((e.into(), self.vocab)),
			}
		}

		for (layout_ref, (id, layout)) in &allocated_shelves.layouts {
			graph.items.push(crate::Item::Layout(layout_ref.cast()));
			match layout.dependencies(*id, &allocated_nodes, layout.causes()) {
				Ok(dependencies) => graph.layout_dependencies.push(dependencies),
				Err(e) => return Err((e.into(), self.vocab)),
			}
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
						let (id, ty) = types_to_build[ty_ref.index()].take().unwrap();
						let (ty, causes) = ty.into_parts();
						match ty.build(id, &self.vocab, &allocated_nodes, dependencies, causes) {
							Ok(built_ty) => {
								built_types[ty_ref.index()] = Some(built_ty);
							}
							Err(e) => return Err((e.into(), self.vocab)),
						}
					}
					crate::Item::Property(prop_ref) => {
						let (id, prop) = properties_to_build[prop_ref.index()].take().unwrap();
						let (prop, causes) = prop.into_parts();
						match prop.build(id, &self.vocab, &allocated_nodes, dependencies, causes) {
							Ok(built_prop) => {
								built_properties[prop_ref.index()] = Some(built_prop);
							}
							Err(e) => return Err((e.into(), self.vocab)),
						}
					}
					crate::Item::Layout(layout_ref) => {
						let (id, layout) = layouts_to_build[layout_ref.index()].take().unwrap();
						let (layout, causes) = layout.into_parts();
						match layout.build(id, &self.vocab, &allocated_nodes, dependencies, causes)
						{
							Ok(built_layout) => {
								built_layouts[layout_ref.index()] = Some(built_layout);
							}
							Err(e) => return Err((e.into(), self.vocab)),
						}
					}
				}
			}
		}

		Ok(Model::from_parts(
			self.vocab,
			allocated_nodes.into_model_nodes(),
			Shelf::new(built_types.into_iter().map(Option::unwrap).collect()),
			Shelf::new(built_properties.into_iter().map(Option::unwrap).collect()),
			Shelf::new(built_layouts.into_iter().map(Option::unwrap).collect()),
		))
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
	pub fn declare_type(
		&mut self,
		id: Id,
		cause: Option<Location<F>>,
		f: impl FnOnce(Id) -> D::Type,
	) where
		F: Ord,
	{
		match self.nodes.get_mut(&id) {
			Some(node) => node.declare_type(cause, f),
			None => {
				self.nodes.insert(id, Node::new_type(id, f(id), cause));
			}
		}
	}

	/// Declare the given `id` as a property.
	pub fn declare_property(
		&mut self,
		id: Id,
		cause: Option<Location<F>>,
		f: impl FnOnce(Id) -> D::Property,
	) where
		F: Ord,
	{
		match self.nodes.get_mut(&id) {
			Some(node) => node.declare_property(cause, f),
			None => {
				self.nodes.insert(id, Node::new_property(id, f(id), cause));
			}
		}
	}

	/// Declare the given `id` as a layout.
	pub fn declare_layout(
		&mut self,
		id: Id,
		cause: Option<Location<F>>,
		f: impl FnOnce(Id) -> D::Layout,
	) where
		F: Ord,
	{
		match self.nodes.get_mut(&id) {
			Some(node) => node.declare_layout(cause, f),
			None => {
				self.nodes.insert(id, Node::new_layout(id, f(id), cause));
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

	pub fn require_type_mut(
		&mut self,
		id: Id,
		cause: Option<Location<F>>,
	) -> Result<&mut WithCauses<D::Type, F>, Error<F>>
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
	) -> Result<&mut WithCauses<D::Property, F>, Error<F>>
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
	) -> Result<&WithCauses<D::Layout, F>, Error<F>>
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
	) -> Result<&mut WithCauses<D::Layout, F>, Error<F>>
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
	) -> Result<node::PropertyOrLayoutField<F, D>, Error<F>>
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
pub struct AllocatedShelves<F, D: Definitions<F>> {
	types: Shelf<Vec<(Id, WithCauses<D::Type, F>)>>,
	properties: Shelf<Vec<(Id, WithCauses<D::Property, F>)>>,
	layouts: Shelf<Vec<(Id, WithCauses<D::Layout, F>)>>,
}

impl<F, D: Definitions<F>> Default for AllocatedShelves<F, D> {
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
	pub fn new<D: Definitions<F>>(
		shelves: &mut AllocatedShelves<F, D>,
		nodes: HashMap<Id, Node<node::Components<F, D>>>,
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
