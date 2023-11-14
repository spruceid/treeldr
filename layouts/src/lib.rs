//! TreeLDR's RDF Layouts are a powerful tool to map structured data to RDF datasets.
//! This library provides core types to define layouts, an abstract syntax to
//! describe layouts and "distillation" functions to serialize/deserialize data
//! using layouts.
//!
//! # Basic usage
//!
//! The following example shows how to create a layout from its abstract syntax
//! representation (using JSON), compile it and use it to serialize an RDF
//! dataset into a structured value.
//!
//! ```
//! use static_iref::iri;
//! use rdf_types::{Quad, Term, Literal, literal::Type};
//! use xsd_types::XSD_STRING;
//! use serde_json::json;
//!
//! // Create a layout builder.
//! let mut builder = treeldr_layouts::abs::Builder::new();
//!
//! // Parse the layout definition, here from JSON.
//! let layout: treeldr_layouts::abs::syntax::Layout = serde_json::from_value(
//!   json!({
//!     "type": "record",
//!     "fields": {
//!       "id": {
//!         "value": {
//!           "layout": { "type": "id" },
//!           "input": "_:self"
//!         }
//!       },
//!       "name": {
//!         "value": { "type": "string" },
//!         "property": "https://schema.org/name"
//!       }
//!     }
//!   })
//! ).unwrap();
//!
//! // Build the layout.
//! let layout_ref = layout.build(&mut builder).unwrap(); // returns a `Ref` to the layout.
//!
//! // Get the compiled layouts collection.
//! let layouts = builder.build();
//!
//! // Create an RDF dataset with a single triple.
//! let dataset: grdf::BTreeDataset = [
//!   Quad(
//!     Term::iri(iri!("https://example.org/#john.smith").to_owned()),
//!     Term::iri(iri!("https://schema.org/name").to_owned()),
//!     Term::Literal(Literal::new("John Smith".to_owned(), Type::Any(XSD_STRING.to_owned()))),
//!     None
//!   )
//! ].into_iter().collect();
//!
//! // Hydrate the dataset to get a structured data value.
//! let value = treeldr_layouts::hydrate(
//!   &layouts,
//!   &dataset,
//!   &layout_ref,
//!   &[Term::iri(iri!("https://example.org/#john.smith").to_owned())]
//! ).unwrap().into_untyped(); // we don't care about types here.
//!
//! // Create a structured data value with the expected result.
//! // Parse the layout definition, here from JSON.
//! let expected: treeldr_layouts::Value = serde_json::from_value(
//!   json!({
//!     "id": "https://example.org/#john.smith",
//!     "name": "John Smith"
//!   })
//! ).unwrap();
//!
//! // Check equality.
//! assert_eq!(value, expected)
//! ```
//!
//! # The `Layout` types
//!
//! Layouts come in several forms:
//!   - [`abs::syntax::Layout`](crate::abs::syntax::Layout): represents a
//!     layout definition in the abstract syntax. In this representation
//!     variables have names and layouts can be nested.
//!   - [`abs::Layout`](crate::abs::Layout): represents an abstract layout with
//!     stripped variable names and flattened layouts. These layouts are
//!     managed by the layout [`Builder`](crate::abs::Builder).
//!   - [`Layout`](crate::Layout): the most optimized and compact form, used
//!     by the distillation functions. Such layouts are stored in a
//!     [`Layouts`](crate::Layouts) collection.
pub mod abs;
pub mod distill;
pub mod format;
pub mod graph;
pub mod layout;
pub mod matching;
pub mod pattern;
pub mod r#ref;
pub mod utils;
pub mod value;

use std::collections::BTreeMap;

pub use distill::{hydrate, hydrate_with};
pub use format::ValueFormat;
pub use graph::{Dataset, Graph};
pub use layout::Layout;
use layout::LayoutType;
pub use matching::Matching;
pub use pattern::Pattern;
pub use r#ref::Ref;
pub use value::{Literal, TypedLiteral, TypedValue, Value};

pub trait GetFromLayouts<C, R>: Sized {
	type Target<'c>
	where
		C: 'c,
		R: 'c;

	fn get_from_layouts<'c>(context: &'c C, r: &Ref<Self, R>) -> Option<Self::Target<'c>>;
}

/// Layout collection.
///
/// Stores compiled layouts definitions, which can then be fetched using the
/// [`Ref<Layout>`](Ref) type.
///
/// Users can create a `Layouts` collection manually, or from the abstract
/// syntax using a layout [`Builder`](abs::Builder).
///
/// The `R` type parameter represents (interpreted) RDF resources. By default,
/// RDF resources are represented using their lexical representation
/// ([`Term`](rdf_types::Term)).
#[derive(Debug)]
pub struct Layouts<R = rdf_types::Term> {
	layouts: BTreeMap<R, Layout<R>>,
}

impl<R> Layouts<R> {
	/// Creates a new empty layout collection.
	pub fn new() -> Self {
		Self {
			layouts: BTreeMap::new(),
		}
	}
}

impl<R> Default for Layouts<R> {
	fn default() -> Self {
		Self::new()
	}
}

impl<R: Ord> Layouts<R> {
	pub fn layout(&self, id: &R) -> Option<&Layout<R>> {
		self.layouts.get(id)
	}

	pub fn get<T: GetFromLayouts<Self, R>>(&self, r: &Ref<T, R>) -> Option<T::Target<'_>> {
		T::get_from_layouts(self, r)
	}
}

impl<R: Clone + Ord> Layouts<R> {
	pub fn insert(&mut self, id: R, layout: Layout<R>) -> (Ref<LayoutType, R>, Option<Layout<R>>) {
		self.insert_with(id, |_| layout)
	}

	pub fn insert_with(
		&mut self,
		id: R,
		builder: impl FnOnce(&Ref<LayoutType, R>) -> Layout<R>,
	) -> (Ref<LayoutType, R>, Option<Layout<R>>) {
		let layout_ref = Ref::new(id.clone());
		let layout = builder(&layout_ref);

		let old_layout = self.layouts.insert(id, layout);

		(layout_ref, old_layout)
	}
}
