use iref::{Iri, IriBuf};

mod feature;
pub mod error;
pub mod source;
pub mod syntax;
pub mod ty;
pub mod prop;
pub mod layout;
pub mod vocab;
pub mod collection;
pub mod node;
mod compile;

pub use feature::Feature;
pub use error::Error;
pub use source::{Source, Cause, Caused};
pub use vocab::{Vocabulary, Id};
pub use collection::{Collection, Ref};
pub use node::Node;
pub use compile::Compile;

/// TreeLDR context.
pub struct Context {
	// Base IRI.
	base_iri: IriBuf,

	/// Vocabulary.
	vocab: Vocabulary,

	/// Nodes.
	nodes: Vec<Node>,

	/// Type definitions.
	types: Collection<ty::Definition>,

	/// Property definitions.
	properties: Collection<prop::Definition>,

	/// Layout definitions.
	layouts: Collection<layout::Definition>
}

impl Context {
	/// Creates a new empty context.
	pub fn new(base_iri: impl Into<IriBuf>) -> Self {
		Self {
			base_iri: base_iri.into(),
			vocab: Vocabulary::new(),
			nodes: Vec::new(),
			types: Collection::new(),
			properties: Collection::new(),
			layouts: Collection::new()
		}
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
	pub fn get(&self, id: Id) -> Option<&Node> {
		self.nodes.get(id.index())
	}

	/// Ensures that the given `id` is defined in the context.
	/// 
	/// If it is not currently defined in the context,
	/// then the given `id` is associated to an [`Node::Unknown`] node.
	pub fn reserve(&mut self, id: Id) {
		if self.nodes.len() <= id.index() {
			for i in self.nodes.len()..(id.index() + 1) {
				self.nodes.push(Node::Unknown(Id(i)))
			}
		}
	}

	/// Inserts the given node to the context.
	/// 
	/// Replaces any previous node with the same [`Node::id`].
	pub fn insert(&mut self, mut node: Node) -> Node {
		let id = node.id(self);
		self.reserve(id);
		std::mem::swap(&mut node, &mut self.nodes[id.index()]);
		node
	}

	/// Returns a reference to the collection of type definitions.
	pub fn types(&self) -> &Collection<ty::Definition> {
		&self.types
	}

	/// Returns a mutable reference to the collection of type definitions.
	pub fn types_mut(&mut self) -> &mut Collection<ty::Definition> {
		&mut self.types
	}

	/// Returns a reference to the collection of property definitions.
	pub fn properties(&self) -> &Collection<prop::Definition> {
		&self.properties
	}

	/// Returns a mutable reference to the collection of property definitions.
	pub fn properties_mut(&mut self) -> &mut Collection<prop::Definition> {
		&mut self.properties
	}

	/// Returns a reference to the collection of layout definitions.
	pub fn layouts(&self) -> &Collection<layout::Definition> {
		&self.layouts
	}

	/// Returns a mutable reference to the collection of layout definitions.
	pub fn layouts_mut(&mut self) -> &mut Collection<layout::Definition> {
		&mut self.layouts
	}

	/// Declare the given `id` as a type.
	/// 
	/// Returns an error if it has already been declared as a non-type node.
	pub fn declare_type(&mut self, id: Id, cause: Option<Cause>) -> Result<Ref<ty::Definition>, Caused<Error>> {
		self.reserve(id);
		match &self.nodes[id.index()] {
			Node::Type(ty_ref) => Ok(*ty_ref),
			Node::Property(prop_ref) => {
				let because = self.properties.get(*prop_ref).unwrap().causes().preferred();
				Err(Caused::new(Error::InvalidNodeType { id, expected: node::Type::Type, found: node::Type::Property, because }, cause))
			},
			Node::Layout(layout_ref) => {
				let because = self.layouts.get(*layout_ref).unwrap().causes().preferred();
				Err(Caused::new(Error::InvalidNodeType { id, expected: node::Type::Type, found: node::Type::Layout, because }, cause))
			},
			Node::Unknown(_) => {
				let ty_ref = self.types.insert(ty::Definition::new(id, cause));
				self.nodes[id.index()] = Node::Type(ty_ref);
				Ok(ty_ref)
			}
		}
	}

	/// Declare the given `id` as a property.
	/// 
	/// Returns an error if it has already been declared as a non-property node.
	pub fn declare_property(&mut self, id: Id, cause: Option<Cause>) -> Result<Ref<prop::Definition>, Caused<Error>> {
		self.reserve(id);
		match &self.nodes[id.index()] {
			Node::Type(ty_ref) => {
				let because = self.types.get(*ty_ref).unwrap().causes().preferred();
				Err(Caused::new(Error::InvalidNodeType { id, expected: node::Type::Property, found: node::Type::Type, because }, cause))
			},
			Node::Property(prop_ref) => Ok(*prop_ref),
			Node::Layout(layout_ref) => {
				let because = self.layouts.get(*layout_ref).unwrap().causes().preferred();
				Err(Caused::new(Error::InvalidNodeType { id, expected: node::Type::Property, found: node::Type::Layout, because }, cause))
			},
			Node::Unknown(_) => {
				let prop_ref = self.properties.insert(prop::Definition::new(id, cause));
				self.nodes[id.index()] = Node::Property(prop_ref);
				Ok(prop_ref)
			}
		}
	}

	/// Declare the given `id` as a layout.
	/// 
	/// Returns an error if it has already been declared as a non-layout node.
	pub fn declare_layout(&mut self, id: Id, cause: Option<Cause>) -> Result<Ref<layout::Definition>, Caused<Error>> {
		self.reserve(id);
		match &self.nodes[id.index()] {
			Node::Type(ty_ref) => {
				let because = self.types.get(*ty_ref).unwrap().causes().preferred();
				Err(Caused::new(Error::InvalidNodeType { id, expected: node::Type::Layout, found: node::Type::Type, because }, cause))
			},
			Node::Property(prop_ref) => {
				let because = self.properties.get(*prop_ref).unwrap().causes().preferred();
				Err(Caused::new(Error::InvalidNodeType { id, expected: node::Type::Layout, found: node::Type::Property, because }, cause))
			},
			Node::Layout(layout_ref) => Ok(*layout_ref),
			Node::Unknown(_) => {
				let layout_ref = self.layouts.insert(layout::Definition::new(id, cause));
				self.nodes[id.index()] = Node::Layout(layout_ref);
				Ok(layout_ref)
			}
		}
	}
}