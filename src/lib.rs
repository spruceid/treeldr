pub mod error;
pub mod source;
pub mod syntax;
pub mod ty;
pub mod prop;
pub mod layout;
pub mod vocab;
pub mod collection;
pub mod node;

pub use error::Error;
pub use source::Source;
pub use vocab::{Vocabulary, Id};
pub use collection::{Collection, Ref};
pub use node::Node;

/// TreeLDR context.
#[derive(Default)]
pub struct Context {
	/// Vocabulary.
	vocab: Vocabulary,

	/// Nodes.
	nodes: Vec<Node>,

	/// Type definitions.
	types: Collection<ty::Definition>,

	/// Property definitions.
	props: Collection<prop::Definition>,

	/// Layout definitions.
	layouts: Collection<layout::Definition>
}

impl Context {
	/// Creates a new empty context.
	pub fn new() -> Self {
		Self::default()
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
		&self.props
	}

	/// Returns a mutable reference to the collection of property definitions.
	pub fn properties_mut(&mut self) -> &mut Collection<prop::Definition> {
		&mut self.props
	}

	/// Returns a reference to the collection of layout definitions.
	pub fn layouts(&self) -> &Collection<layout::Definition> {
		&self.layouts
	}

	/// Returns a mutable reference to the collection of layout definitions.
	pub fn layouts_mut(&mut self) -> &mut Collection<layout::Definition> {
		&mut self.layouts
	}
}