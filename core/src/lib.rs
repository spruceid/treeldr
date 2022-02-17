use iref::{Iri, IriBuf};

pub mod build;
mod cause;
pub mod collection;
mod doc;
pub mod error;
mod feature;
pub mod layout;
pub mod node;
pub mod prop;
pub mod source;
pub mod syntax;
pub mod ty;
pub mod vocab;

pub use build::Build;
pub use cause::*;
pub use collection::{Collection, Ref};
pub use doc::Documentation;
pub use error::Error;
pub use feature::Feature;
pub use node::Node;
pub use vocab::{Id, Vocabulary};

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
	layouts: Collection<layout::Definition>,
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
			layouts: Collection::new(),
		}
	}

	pub fn define_native_type(
		&mut self,
		iri: IriBuf,
		native_layout: layout::Native,
		cause: Option<Cause>,
	) -> Result<Ref<layout::Definition>, Caused<layout::Mismatch>> {
		let id = self.vocabulary_mut().insert(iri);
		let ty_ref = self.declare_type(id, cause);

		let layout_ref = self.declare_layout(id, cause);
		let layout = self.layouts.get_mut(layout_ref).unwrap();
		layout.declare_type(ty_ref, cause).unwrap();
		layout.declare_native(native_layout, cause)?;

		Ok(layout_ref)
	}

	pub fn define_xml_types(&mut self) -> Result<(), Caused<layout::Mismatch>> {
		self.define_native_type(
			IriBuf::new("http://www.w3.org/2001/XMLSchema#boolean").unwrap(),
			layout::Native::Boolean,
			None,
		)?;
		self.define_native_type(
			IriBuf::new("http://www.w3.org/2001/XMLSchema#int").unwrap(),
			layout::Native::Integer,
			None,
		)?;
		self.define_native_type(
			IriBuf::new("http://www.w3.org/2001/XMLSchema#integer").unwrap(),
			layout::Native::Integer,
			None,
		)?;
		self.define_native_type(
			IriBuf::new("http://www.w3.org/2001/XMLSchema#positiveInteger").unwrap(),
			layout::Native::PositiveInteger,
			None,
		)?;
		self.define_native_type(
			IriBuf::new("http://www.w3.org/2001/XMLSchema#float").unwrap(),
			layout::Native::Float,
			None,
		)?;
		self.define_native_type(
			IriBuf::new("http://www.w3.org/2001/XMLSchema#double").unwrap(),
			layout::Native::Double,
			None,
		)?;
		self.define_native_type(
			IriBuf::new("http://www.w3.org/2001/XMLSchema#string").unwrap(),
			layout::Native::String,
			None,
		)?;
		self.define_native_type(
			IriBuf::new("http://www.w3.org/2001/XMLSchema#time").unwrap(),
			layout::Native::Time,
			None,
		)?;
		self.define_native_type(
			IriBuf::new("http://www.w3.org/2001/XMLSchema#date").unwrap(),
			layout::Native::Date,
			None,
		)?;
		self.define_native_type(
			IriBuf::new("http://www.w3.org/2001/XMLSchema#dateTime").unwrap(),
			layout::Native::DateTime,
			None,
		)?;
		self.define_native_type(
			IriBuf::new("http://www.w3.org/2001/XMLSchema#anyURI").unwrap(),
			layout::Native::Uri,
			None,
		)?;

		Ok(())
	}

	pub fn define_reference_layout(
		&mut self,
		arg_layout_ref: Ref<layout::Definition>,
		cause: Option<Cause>,
	) -> Result<Ref<layout::Definition>, Caused<layout::Mismatch>> {
		let arg_layout = self.layouts().get(arg_layout_ref).unwrap();
		let arg_iri = self.vocabulary().get(arg_layout.id()).unwrap();
		let arg_pct_iri =
			pct_str::PctString::encode(arg_iri.as_str().chars(), pct_str::URIReserved);
		let iri = IriBuf::from_string(format!(
			"http://schema.treeldr.org/Reference_{}",
			arg_pct_iri
		))
		.unwrap();
		self.define_native_type(iri, layout::Native::Reference(arg_layout_ref), cause)
	}

	pub fn check(&self) -> Result<(), Caused<Error>> {
		for (_, layout) in self.layouts.iter() {
			layout.check(self)?;
		}

		Ok(())
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

	pub fn nodes(&self) -> vocab::Iter<Node> {
		self.nodes.iter()
	}

	pub fn nodes_mut(&mut self) -> vocab::IterMut<Node> {
		self.nodes.iter_mut()
	}

	/// Inserts the given node to the context.
	///
	/// Replaces any previous node with the same [`Node::id`].
	pub fn insert(&mut self, node: Node) -> Option<Node> {
		self.nodes.insert(node.id(), node)
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
	pub fn declare_type(&mut self, id: Id, cause: Option<Cause>) -> Ref<ty::Definition> {
		match self.nodes.get_mut(id) {
			Some(node) => match node.as_type() {
				Some(ty_ref) => ty_ref,
				None => {
					let ty_ref = self.types.insert(ty::Definition::new(id, cause));
					node.declare_type(ty_ref);
					ty_ref
				}
			},
			None => {
				let ty_ref = self.types.insert(ty::Definition::new(id, cause));
				self.nodes.insert(id, Node::new_type(id, ty_ref));
				ty_ref
			}
		}
	}

	/// Requires the given type to be declared.
	///
	/// Returns an error if no node with the given `id` is declared,
	/// or if it is not a type.
	pub fn require_type(
		&self,
		id: Id,
		source: Option<syntax::Location>,
	) -> Result<Ref<ty::Definition>, Caused<Error>> {
		match self.get(id) {
			None => Err(Caused::new(
				Error::UnknownNode {
					id,
					expected_ty: Some(node::Type::Type),
				},
				source.map(Cause::Explicit),
			)),
			Some(node) => match node.as_type() {
				Some(ty_ref) => Ok(ty_ref),
				None => Err(Caused::new(
					Error::InvalidNodeType {
						id,
						expected: node::Type::Type,
						found: node.caused_types(self),
					},
					source.map(Cause::Explicit),
				)),
			},
		}
	}

	/// Declare the given `id` as a property.
	pub fn declare_property(&mut self, id: Id, cause: Option<Cause>) -> Ref<prop::Definition> {
		match self.nodes.get_mut(id) {
			Some(node) => match node.as_property() {
				Some(prop_ref) => prop_ref,
				None => {
					let prop_ref = self.properties.insert(prop::Definition::new(id, cause));
					node.declare_property(prop_ref);
					prop_ref
				}
			},
			None => {
				let prop_ref = self.properties.insert(prop::Definition::new(id, cause));
				self.nodes.insert(id, Node::new_property(id, prop_ref));
				prop_ref
			}
		}
	}

	/// Requires the given property to be declared.
	///
	/// Returns an error if no node with the given `id` is declared,
	/// or if it is not a property.
	pub fn require_property(
		&self,
		id: Id,
		source: Option<syntax::Location>,
	) -> Result<Ref<prop::Definition>, Caused<Error>> {
		match self.get(id) {
			None => Err(Caused::new(
				Error::UnknownNode {
					id,
					expected_ty: Some(node::Type::Property),
				},
				source.map(Cause::Explicit),
			)),
			Some(node) => match node.as_property() {
				Some(prop_ref) => Ok(prop_ref),
				None => Err(Caused::new(
					Error::InvalidNodeType {
						id,
						expected: node::Type::Property,
						found: node.caused_types(self),
					},
					source.map(Cause::Explicit),
				)),
			},
		}
	}

	/// Declare the given `id` as a layout.
	pub fn declare_layout(&mut self, id: Id, cause: Option<Cause>) -> Ref<layout::Definition> {
		match self.nodes.get_mut(id) {
			Some(node) => match node.as_layout() {
				Some(layout_ref) => layout_ref,
				None => {
					let layout_ref = self.layouts.insert(layout::Definition::new(id, cause));
					node.declare_layout(layout_ref);
					layout_ref
				}
			},
			None => {
				let layout_ref = self.layouts.insert(layout::Definition::new(id, cause));
				self.nodes.insert(id, Node::new_layout(id, layout_ref));
				layout_ref
			}
		}
	}

	/// Requires the given layout to be declared.
	///
	/// Returns an error if no node with the given `id` is declared,
	/// or if it is not a layout.
	pub fn require_layout(
		&self,
		id: Id,
		cause: Option<Cause>,
	) -> Result<Ref<layout::Definition>, Caused<Error>> {
		match self.get(id) {
			None => Err(Caused::new(
				Error::UnknownNode {
					id,
					expected_ty: Some(node::Type::Layout),
				},
				cause,
			)),
			Some(node) => match node.as_layout() {
				Some(layout_ref) => Ok(layout_ref),
				None => Err(Caused::new(
					Error::InvalidNodeType {
						id,
						expected: node::Type::Layout,
						found: node.caused_types(self),
					},
					cause,
				)),
			},
		}
	}
}
