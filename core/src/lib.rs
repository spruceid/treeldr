use derivative::Derivative;
use shelves::Shelf;
use std::collections::HashMap;
use std::fmt;

pub mod build;
mod cause;
mod doc;
mod feature;
pub mod layout;
mod maybe_set;
pub mod node;
pub mod prop;
pub mod reporting;
pub mod ty;
pub mod utils;
pub use treeldr_vocab as vocab;

pub use cause::*;
pub use doc::Documentation;
pub use feature::Feature;
pub use maybe_set::*;
pub use node::Node;
pub use vocab::{Id, Vocabulary};

/// TreeLDR model.
#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct Model<F> {
	/// Vocabulary.
	vocab: Vocabulary,

	/// Nodes.
	nodes: HashMap<Id, Node<F>>,

	/// Type definitions.
	types: Shelf<Vec<ty::Definition<F>>>,

	/// Property definitions.
	properties: Shelf<Vec<prop::Definition<F>>>,

	/// Layout definitions.
	layouts: Shelf<Vec<layout::Definition<F>>>,
}

impl<F> Model<F> {
	/// Creates a new empty context.
	pub fn new() -> Self {
		Self::default()
	}

	pub fn with_vocabulary(vocab: Vocabulary) -> Self {
		Self {
			vocab,
			nodes: HashMap::new(),
			types: Shelf::default(),
			properties: Shelf::default(),
			layouts: Shelf::default(),
		}
	}

	pub fn from_parts(
		vocab: Vocabulary,
		nodes: HashMap<Id, Node<F>>,
		types: Shelf<Vec<ty::Definition<F>>>,
		properties: Shelf<Vec<prop::Definition<F>>>,
		layouts: Shelf<Vec<layout::Definition<F>>>,
	) -> Self {
		Self {
			vocab,
			nodes,
			types,
			properties,
			layouts,
		}
	}

	// pub fn define_reference_layout(
	// 	&mut self,
	// 	arg_layout_ref: Ref<layout::Definition<F>>,
	// 	cause: Option<Location<F>>,
	// ) -> Result<Ref<layout::Definition<F>>, Caused<layout::Mismatch<F>, F>> where F: Clone + Ord {
	// 	let arg_layout = self.layouts().get(arg_layout_ref).unwrap();
	// 	let arg_iri = self.vocabulary().get(arg_layout.id()).unwrap();
	// 	let arg_pct_iri =
	// 		pct_str::PctString::encode(arg_iri.as_str().chars(), pct_str::URIReserved);
	// 	let iri = IriBuf::from_string(format!(
	// 		"http://schema.treeldr.org/Reference_{}",
	// 		arg_pct_iri
	// 	))
	// 	.unwrap();
	// 	self.define_native_type(iri, layout::Native::Reference(arg_layout_ref), cause)
	// }

	// pub fn check(&self) -> Result<(), Error<F>> where F: Clone {
	// 	for (_, layout) in self.layouts.iter() {
	// 		layout.check(self)?;
	// 	}

	// 	Ok(())
	// }

	/// Returns a reference to the vocabulary.
	pub fn vocabulary(&self) -> &Vocabulary {
		&self.vocab
	}

	/// Returns a mutable reference to the vocabulary.
	pub fn vocabulary_mut(&mut self) -> &mut Vocabulary {
		&mut self.vocab
	}

	/// Returns the node associated to the given `Id`, if any.
	pub fn get(&self, id: Id) -> Option<&Node<F>> {
		self.nodes.get(&id)
	}

	/// Returns a mutable reference to the node associated to the given `Id`, if any.
	pub fn get_mut(&mut self, id: Id) -> Option<&mut Node<F>> {
		self.nodes.get_mut(&id)
	}

	pub fn nodes(&self) -> impl Iterator<Item = (Id, &Node<F>)> {
		self.nodes.iter().map(|(id, node)| (*id, node))
	}

	pub fn nodes_mut(&mut self) -> impl Iterator<Item = (Id, &mut Node<F>)> {
		self.nodes.iter_mut().map(|(id, node)| (*id, node))
	}

	/// Inserts the given node to the context.
	///
	/// Replaces any previous node with the same [`Node::id`].
	pub fn insert(&mut self, node: Node<F>) -> Option<Node<F>> {
		self.nodes.insert(node.id(), node)
	}

	/// Returns a reference to the collection of type definitions.
	pub fn types(&self) -> &Shelf<Vec<ty::Definition<F>>> {
		&self.types
	}

	/// Returns a mutable reference to the collection of type definitions.
	pub fn types_mut(&mut self) -> &mut Shelf<Vec<ty::Definition<F>>> {
		&mut self.types
	}

	/// Returns a reference to the collection of property definitions.
	pub fn properties(&self) -> &Shelf<Vec<prop::Definition<F>>> {
		&self.properties
	}

	/// Returns a mutable reference to the collection of property definitions.
	pub fn properties_mut(&mut self) -> &mut Shelf<Vec<prop::Definition<F>>> {
		&mut self.properties
	}

	/// Returns a reference to the collection of layout definitions.
	pub fn layouts(&self) -> &Shelf<Vec<layout::Definition<F>>> {
		&self.layouts
	}

	/// Returns a mutable reference to the collection of layout definitions.
	pub fn layouts_mut(&mut self) -> &mut Shelf<Vec<layout::Definition<F>>> {
		&mut self.layouts
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
