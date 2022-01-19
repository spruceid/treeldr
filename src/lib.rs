use iref::{Iri, IriBuf};

mod feature;
pub mod error;
mod doc;
pub mod source;
pub mod syntax;
pub mod ty;
pub mod prop;
pub mod layout;
pub mod vocab;
pub mod collection;
pub mod node;
pub mod compile;

pub use feature::Feature;
pub use error::Error;
pub use doc::Documentation;
pub use source::{Source, Cause, Caused};
pub use vocab::{Vocabulary, Id};
pub use collection::{Collection, Ref};
pub use node::Node;
pub use compile::Compile;

/// TreeLDR model.
pub struct Model {
	/// Base IRI.
	base_iri: IriBuf,

	/// Vocabulary.
	vocab: Vocabulary,

	/// Nodes.
	nodes: vocab::Map<Node>,

	/// Type definitions.
	types: Collection<ty::Definition>,

	/// Property definitions.
	properties: Collection<prop::Definition>,

	/// Layout definitions.
	layouts: Collection<layout::Definition>
}

impl Model {
	/// Creates a new empty context.
	pub fn new(base_iri: impl Into<IriBuf>) -> Self {
		Self {
			base_iri: base_iri.into(),
			vocab: Vocabulary::new(),
			nodes: vocab::Map::new(),
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
		self.nodes.get(id)
	}

	/// Inserts the given node to the context.
	/// 
	/// Replaces any previous node with the same [`Node::id`].
	pub fn insert(&mut self, node: Node) -> Option<Node> {
		let id = node.id(self);
		self.nodes.insert(id, node)
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
		match self.nodes.get_mut(id) {
			Some(Node::Type(ty_ref)) => Ok(*ty_ref),
			Some(Node::Property(prop_ref)) => {
				let because = self.properties.get(*prop_ref).unwrap().causes().preferred();
				Err(Caused::new(Error::InvalidNodeType { id, expected: node::Type::Type, found: Some(node::Type::Property), because }, cause))
			},
			Some(Node::Layout(layout_ref)) => {
				let because = self.layouts.get(*layout_ref).unwrap().causes().preferred();
				Err(Caused::new(Error::InvalidNodeType { id, expected: node::Type::Type, found: Some(node::Type::Layout), because }, cause))
			},
			_ => {
				let ty_ref = self.types.insert(ty::Definition::new(id, cause));
				self.nodes.insert(id, Node::Type(ty_ref));
				Ok(ty_ref)
			}
		}
	}

	/// Requires the given type to be declared.
	/// 
	/// Returns an error if no node with the given `id` is declared,
	/// or if it is not a type.
	pub fn require_type(&self, id: Id, source: Option<Source>) -> Result<Ref<ty::Definition>, Caused<Error>> {
		match self.get(id) {
			None => Err(Caused::new(Error::UnknownNode { id, expected_ty: Some(node::Type::Type) }, source.map(Cause::Explicit))),
			Some(Node::Type(ty_ref)) => Ok(*ty_ref),
			Some(Node::Property(prop_ref)) => {
				let because = self.properties.get(*prop_ref).unwrap().causes().preferred();
				Err(Caused::new(Error::InvalidNodeType { id, expected: node::Type::Type, found: Some(node::Type::Property), because }, source.map(Cause::Explicit)))
			}
			Some(Node::Layout(layout_ref)) => {
				let because = self.layouts.get(*layout_ref).unwrap().causes().preferred();
				Err(Caused::new(Error::InvalidNodeType { id, expected: node::Type::Type, found: Some(node::Type::Layout), because }, source.map(Cause::Explicit)))
			}
			Some(Node::Unknown(_)) => {
				Err(Caused::new(Error::InvalidNodeType { id, expected: node::Type::Type, found: None, because: None }, source.map(Cause::Explicit)))
			}
		}
	}

	/// Declare the given `id` as a property.
	/// 
	/// Returns an error if it has already been declared as a non-property node.
	pub fn declare_property(&mut self, id: Id, cause: Option<Cause>) -> Result<Ref<prop::Definition>, Caused<Error>> {
		match self.nodes.get_mut(id) {
			Some(Node::Type(ty_ref)) => {
				let because = self.types.get(*ty_ref).unwrap().causes().preferred();
				Err(Caused::new(Error::InvalidNodeType { id, expected: node::Type::Property, found: Some(node::Type::Type), because }, cause))
			},
			Some(Node::Property(prop_ref)) => Ok(*prop_ref),
			Some(Node::Layout(layout_ref)) => {
				let because = self.layouts.get(*layout_ref).unwrap().causes().preferred();
				Err(Caused::new(Error::InvalidNodeType { id, expected: node::Type::Property, found: Some(node::Type::Layout), because }, cause))
			},
			_ => {
				let prop_ref = self.properties.insert(prop::Definition::new(id, cause));
				self.nodes.insert(id, Node::Property(prop_ref));
				Ok(prop_ref)
			}
		}
	}

	/// Requires the given property to be declared.
	/// 
	/// Returns an error if no node with the given `id` is declared,
	/// or if it is not a property.
	pub fn require_property(&self, id: Id, source: Option<Source>) -> Result<Ref<prop::Definition>, Caused<Error>> {
		match self.get(id) {
			None => Err(Caused::new(Error::UnknownNode { id, expected_ty: Some(node::Type::Property) }, source.map(Cause::Explicit))),
			Some(Node::Type(ty_ref)) => {
				let because = self.types.get(*ty_ref).unwrap().causes().preferred();
				Err(Caused::new(Error::InvalidNodeType { id, expected: node::Type::Property, found: Some(node::Type::Type), because }, source.map(Cause::Explicit)))
			},
			Some(Node::Property(prop_ref)) => Ok(*prop_ref),
			Some(Node::Layout(layout_ref)) => {
				let because = self.layouts.get(*layout_ref).unwrap().causes().preferred();
				Err(Caused::new(Error::InvalidNodeType { id, expected: node::Type::Property, found: Some(node::Type::Layout), because }, source.map(Cause::Explicit)))
			}
			Some(Node::Unknown(_)) => {
				Err(Caused::new(Error::InvalidNodeType { id, expected: node::Type::Property, found: None, because: None }, source.map(Cause::Explicit)))
			}
		}
	}

	/// Declare the given `id` as a layout.
	/// 
	/// Returns an error if it has already been declared as a non-layout node.
	pub fn declare_layout(&mut self, id: Id, cause: Option<Cause>) -> Result<Ref<layout::Definition>, Caused<Error>> {
		match self.nodes.get_mut(id) {
			Some(Node::Type(ty_ref)) => {
				let because = self.types.get(*ty_ref).unwrap().causes().preferred();
				Err(Caused::new(Error::InvalidNodeType { id, expected: node::Type::Layout, found: Some(node::Type::Type), because }, cause))
			},
			Some(Node::Property(prop_ref)) => {
				let because = self.properties.get(*prop_ref).unwrap().causes().preferred();
				Err(Caused::new(Error::InvalidNodeType { id, expected: node::Type::Layout, found: Some(node::Type::Property), because }, cause))
			},
			Some(Node::Layout(layout_ref)) => Ok(*layout_ref),
			_ => {
				let layout_ref = self.layouts.insert(layout::Definition::new(id, cause));
				self.nodes.insert(id, Node::Layout(layout_ref));
				Ok(layout_ref)
			}
		}
	}

	/// Requires the given layout to be declared.
	/// 
	/// Returns an error if no node with the given `id` is declared,
	/// or if it is not a layout.
	pub fn require_layout(&self, id: Id, source: Option<Source>) -> Result<Ref<layout::Definition>, Caused<Error>> {
		match self.get(id) {
			None => Err(Caused::new(Error::UnknownNode { id, expected_ty: Some(node::Type::Layout) }, source.map(Cause::Explicit))),
			Some(Node::Type(ty_ref)) => {
				let because = self.types.get(*ty_ref).unwrap().causes().preferred();
				Err(Caused::new(Error::InvalidNodeType { id, expected: node::Type::Layout, found: Some(node::Type::Type), because }, source.map(Cause::Explicit)))
			},
			Some(Node::Property(prop_ref)) => {
				let because = self.properties.get(*prop_ref).unwrap().causes().preferred();
				Err(Caused::new(Error::InvalidNodeType { id, expected: node::Type::Layout, found: Some(node::Type::Property), because }, source.map(Cause::Explicit)))
			},
			Some(Node::Layout(layout_ref)) => Ok(*layout_ref),
			Some(Node::Unknown(_)) => {
				Err(Caused::new(Error::InvalidNodeType { id, expected: node::Type::Layout, found: None, because: None }, source.map(Cause::Explicit)))
			}
		}
	}
}