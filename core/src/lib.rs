use derivative::Derivative;
use shelves::Shelf;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fmt;

pub use shelves::Ref;

mod doc;
pub mod error;
mod feature;
pub mod layout;
mod meta_option;
pub mod metadata;
pub mod name;
pub mod node;
pub mod prop;
pub mod reporting;
mod simplify;
pub mod to_rdf;
pub mod ty;
pub mod utils;
pub mod value;
pub mod vocab;

pub use doc::Documentation;
pub use error::Error;
pub use feature::Feature;
pub use meta_option::MetaOption;
pub use metadata::Metadata;
pub use name::Name;
pub use node::Node;
pub use value::Value;
pub use vocab::{BlankIdIndex, Id, IriIndex};

/// TreeLDR model.
#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct Model<M> {
	/// Nodes.
	nodes: BTreeMap<Id, Node<M>>,

	/// Type definitions.
	types: Shelf<Vec<ty::Definition<M>>>,

	/// Property definitions.
	properties: Shelf<Vec<prop::Definition<M>>>,

	/// Layout definitions.
	layouts: Shelf<Vec<layout::Definition<M>>>,
}

impl<M> Model<M> {
	/// Creates a new empty context.
	pub fn new() -> Self {
		Self::default()
	}

	pub fn from_parts(
		nodes: BTreeMap<Id, Node<M>>,
		types: Shelf<Vec<ty::Definition<M>>>,
		properties: Shelf<Vec<prop::Definition<M>>>,
		layouts: Shelf<Vec<layout::Definition<M>>>,
	) -> Self {
		Self {
			nodes,
			types,
			properties,
			layouts,
		}
	}

	pub fn can_be_reference_layout(
		&self,
		map: &mut HashMap<Ref<layout::Definition<M>>, bool>,
		r: Ref<layout::Definition<M>>,
	) -> bool {
		match map.get(&r).cloned() {
			Some(b) => b,
			None => {
				let b = self.layouts.get(r).unwrap().can_be_reference(map, self);
				map.insert(r, b);
				b
			}
		}
	}

	/// Returns the node associated to the given `id`, if any.
	pub fn get(&self, id: Id) -> Option<&Node<M>> {
		self.nodes.get(&id)
	}

	/// Returns a mutable reference to the node associated to the given `id`, if any.
	pub fn get_mut(&mut self, id: Id) -> Option<&mut Node<M>> {
		self.nodes.get_mut(&id)
	}

	pub fn nodes(&self) -> impl Iterator<Item = (Id, &Node<M>)> {
		self.nodes.iter().map(|(i, n)| (*i, n))
	}

	pub fn nodes_mut(&mut self) -> impl Iterator<Item = (Id, &mut Node<M>)> {
		self.nodes.iter_mut().map(|(i, n)| (*i, n))
	}

	/// Inserts the given node to the context.
	///
	/// Replaces any previous node with the same [`Node::id`].
	pub fn insert(&mut self, node: Node<M>) -> Option<Node<M>> {
		self.nodes.insert(node.id(), node)
	}

	/// Returns a reference to the collection of type definitions.
	pub fn types(&self) -> &Shelf<Vec<ty::Definition<M>>> {
		&self.types
	}

	/// Returns a mutable reference to the collection of type definitions.
	pub fn types_mut(&mut self) -> &mut Shelf<Vec<ty::Definition<M>>> {
		&mut self.types
	}

	/// Returns a reference to the collection of property definitions.
	pub fn properties(&self) -> &Shelf<Vec<prop::Definition<M>>> {
		&self.properties
	}

	/// Returns a mutable reference to the collection of property definitions.
	pub fn properties_mut(&mut self) -> &mut Shelf<Vec<prop::Definition<M>>> {
		&mut self.properties
	}

	/// Returns a reference to the collection of layout definitions.
	pub fn layouts(&self) -> &Shelf<Vec<layout::Definition<M>>> {
		&self.layouts
	}

	/// Returns a mutable reference to the collection of layout definitions.
	pub fn layouts_mut(&mut self) -> &mut Shelf<Vec<layout::Definition<M>>> {
		&mut self.layouts
	}

	pub fn require(&self, id: Id, expected_ty: Option<node::Type>) -> Result<&Node<M>, Error<M>> {
		self.get(id)
			.ok_or_else(|| error::NodeUnknown { id, expected_ty }.into())
	}

	pub fn require_layout(&self, id: Id) -> Result<Ref<layout::Definition<M>>, Error<M>>
	where
		M: Clone,
	{
		self.require(id, Some(node::Type::Layout))?.require_layout()
	}
}

pub(crate) trait SubstituteReferences<M> {
	fn substitute_references<I, T, P, L>(&mut self, sub: &ReferenceSubstitution<I, T, P, L>)
	where
		I: Fn(Id) -> Id,
		T: Fn(Ref<ty::Definition<M>>) -> Ref<ty::Definition<M>>,
		P: Fn(Ref<prop::Definition<M>>) -> Ref<prop::Definition<M>>,
		L: Fn(Ref<layout::Definition<M>>) -> Ref<layout::Definition<M>>;
}

impl<M> SubstituteReferences<M> for Model<M> {
	fn substitute_references<I, T, P, L>(&mut self, sub: &ReferenceSubstitution<I, T, P, L>)
	where
		I: Fn(Id) -> Id,
		T: Fn(Ref<ty::Definition<M>>) -> Ref<ty::Definition<M>>,
		P: Fn(Ref<prop::Definition<M>>) -> Ref<prop::Definition<M>>,
		L: Fn(Ref<layout::Definition<M>>) -> Ref<layout::Definition<M>>,
	{
		for (_, ty) in self.types.iter_mut() {
			ty.substitute_references(sub);
		}

		for (_, prop) in self.properties.iter_mut() {
			prop.substitute_references(sub);
		}

		for (_, layout) in self.layouts.iter_mut() {
			layout.substitute_references(sub);
		}
	}
}

pub struct ReferenceSubstitution<I, T, P, L> {
	ids: I,
	types: T,
	properties: P,
	layouts: L,
}

impl<I, T, P, L> ReferenceSubstitution<I, T, P, L> {
	pub fn new(ids: I, types: T, properties: P, layouts: L) -> Self {
		Self {
			ids,
			types,
			properties,
			layouts,
		}
	}

	pub fn id(&self, id: Id) -> Id
	where
		I: Fn(Id) -> Id,
	{
		(self.ids)(id)
	}

	pub fn ty<M>(&self, r: Ref<ty::Definition<M>>) -> Ref<ty::Definition<M>>
	where
		T: Fn(Ref<ty::Definition<M>>) -> Ref<ty::Definition<M>>,
	{
		(self.types)(r)
	}

	pub fn property<M>(&self, r: Ref<prop::Definition<M>>) -> Ref<prop::Definition<M>>
	where
		P: Fn(Ref<prop::Definition<M>>) -> Ref<prop::Definition<M>>,
	{
		(self.properties)(r)
	}

	pub fn layout<M>(&self, r: Ref<layout::Definition<M>>) -> Ref<layout::Definition<M>>
	where
		L: Fn(Ref<layout::Definition<M>>) -> Ref<layout::Definition<M>>,
	{
		(self.layouts)(r)
	}
}

pub struct WithModel<'m, 't, T: ?Sized, F> {
	model: &'m Model<F>,
	value: &'t T,
}

pub trait DisplayWithModel<F> {
	fn fmt(&self, model: &Model<F>, f: &mut fmt::Formatter) -> fmt::Result;

	fn with_model<'m>(&self, model: &'m Model<F>) -> WithModel<'m, '_, Self, F> {
		WithModel { model, value: self }
	}
}

impl<'m, 't, T: DisplayWithModel<F>, F> fmt::Display for WithModel<'m, 't, T, F> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.value.fmt(self.model, f)
	}
}
