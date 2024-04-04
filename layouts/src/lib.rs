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
//! use rdf_types::{Quad, Term, Literal, LiteralType, dataset::IndexedBTreeDataset};
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
//! let dataset: IndexedBTreeDataset = [
//!   Quad(
//!     Term::iri(iri!("https://example.org/#john.smith").to_owned()),
//!     Term::iri(iri!("https://schema.org/name").to_owned()),
//!     Term::Literal(Literal::new("John Smith".to_owned(), LiteralType::Any(XSD_STRING.to_owned()))),
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
#![allow(rustdoc::redundant_explicit_links)]
pub mod abs;
pub mod distill;
pub mod format;
pub mod graph;
pub mod layout;
pub mod matching;
pub mod pattern;
mod prelude;
pub mod preset;
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
pub use prelude::Prelude;
pub use preset::PresetLayout;
pub use r#ref::{DerefResource, Ref};
use rdf_types::Term;
pub use value::{Literal, TypedLiteral, TypedValue, Value};

/// Layout registry.
///
/// Any collection of layout, used by the serialization and deserialization
/// functions.
///
/// This trait is notably implemented by [`Layouts`], returned after compiling
/// layouts from the abstract syntax, and by [`Prelude`] which provides all the
/// layouts defined by TreeLDR's prelude (`tldr:bool`, `tldr:string`, `tldr:u8`,
/// etc.).
///
/// Registries can be combined together using the [`with`] method.
///
/// [`with`]: LayoutRegistry::with
///
/// # Example
///
/// ```
/// use serde_json::json;
/// use treeldr_layouts::{abs, LayoutRegistry, distill::dehydrate, Prelude};
/// let mut builder = abs::Builder::new();
/// let layout: abs::syntax::Layout = serde_json::from_value(json!({
///   "type": "record",
///   "prefixes": { "tldr": "https://treeldr.org/layouts#" },
///   "fields": {
///     "name": {
///       "value": "tldr:string", // This layout is defined in the prelude.
///       "property": "https://example.org/#name"
///     }
///   }
/// })).unwrap();
/// let layout_ref = layout.build(&mut builder).unwrap();
/// let layouts = builder.build();
/// let input: treeldr_layouts::Value = serde_json::from_value(json!({
///   "name": "John Smith"
/// })).unwrap();
/// dehydrate(
///   layouts.with(Prelude), // Add the prelude layouts.
///   &input,
///   &layout_ref,
///   Default::default()
/// );
/// ```
pub trait LayoutRegistry<R = Term> {
	/// Returns the layout definition associated to the given layout identifier.
	fn get(&self, id: &Ref<LayoutType, R>) -> Option<&Layout<R>>;

	/// Checks if this registry contains the definition for the given layout.
	fn contains(&self, id: &Ref<LayoutType, R>) -> bool {
		self.get(id).is_some()
	}

	/// Returns the union of this registry with `other`.
	fn with<T: LayoutRegistry<R>>(&self, other: T) -> LayoutRegistryUnion<T, &Self> {
		LayoutRegistryUnion(other, self)
	}

	/// Returns the union of this registry with `other`.
	///
	/// Same as `with`, but consumes `self`.
	fn into_with<T: LayoutRegistry<R>>(self, other: T) -> LayoutRegistryUnion<T, Self>
	where
		Self: Sized,
	{
		LayoutRegistryUnion(other, self)
	}
}

impl<'a, R, T: LayoutRegistry<R>> LayoutRegistry<R> for &'a T {
	fn get(&self, id: &Ref<LayoutType, R>) -> Option<&Layout<R>> {
		T::get(*self, id)
	}

	fn contains(&self, id: &Ref<LayoutType, R>) -> bool {
		T::contains(*self, id)
	}
}

impl<R, T: LayoutRegistry<R>> LayoutRegistry<R> for Option<T> {
	fn get(&self, id: &Ref<LayoutType, R>) -> Option<&Layout<R>> {
		self.as_ref()?.get(id)
	}

	fn contains(&self, id: &Ref<LayoutType, R>) -> bool {
		self.as_ref().is_some_and(|r| r.contains(id))
	}
}

/// Layout collection.
///
/// Stores compiled layouts definitions, which can then be fetched using the
/// [`Ref<LayoutType>`](Ref) type.
///
/// Users can create a `Layouts` collection manually, or from the abstract
/// syntax using a layout [`Builder`](abs::Builder).
///
/// The `R` type parameter represents (interpreted) RDF resources.
/// The default resource identifier is [`Term`], meaning that the resource is
/// identified by its lexical RDF representation (an IRI, a blank node
/// identifier or a literal value). This default parameter is easy to use but
/// beware of the following:
///   - A resource may have more than one lexical representation. Hence the
///     [`Term`] type is not adequate as a unique resource identifier.
///   - A term is basically a text string, it requires allocation when created
///     and cloned, and comparison is done in linear time (`O(n)`).
/// For these reasons, it is advised to use a more optimized/unique identifier
/// type for resources, using [`Vocabulary`](rdf_types::Vocabulary) to store
/// the actual lexical representations and
/// [`Interpretation`](rdf_types::Interpretation) to map lexical representations
/// to resources.
///
/// [`Term`]: rdf_types::Term
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

	/// Returns an iterator over all the layout definitions stored in this
	/// collection.
	pub fn iter(&self) -> LayoutsIter<R> {
		LayoutsIter(self.layouts.iter())
	}
}

impl<R> Default for Layouts<R> {
	fn default() -> Self {
		Self::new()
	}
}

impl<R: Ord> Layouts<R> {
	/// Returns the layout definition associated to the untyped resource
	/// identifier `id`.
	pub fn layout(&self, id: &R) -> Option<&Layout<R>> {
		self.layouts.get(id)
	}

	/// Gets the definition associated to the given type resource identifier.
	pub fn get<T>(&self, r: &Ref<T, R>) -> Option<<Self as DerefResource<T, R>>::Target<'_>>
	where
		Self: DerefResource<T, R>,
	{
		self.deref_resource(r)
	}
}

impl<R: Clone + Ord> Layouts<R> {
	/// Sets the layout definition for the resource identified by `id`.
	///
	/// Returns the typed identifier for the layout alongside with the previous
	/// layout definition for `id`, if any.
	pub fn insert(&mut self, id: R, layout: Layout<R>) -> (Ref<LayoutType, R>, Option<Layout<R>>) {
		self.insert_with(id, |_| layout)
	}

	/// Sets the layout definition for the resource identified by `id` using a
	/// function. The function will be called with a typed identifier to the
	/// layout.
	///
	/// Returns the typed identifier for the layout alongside with the previous
	/// layout definition for `id`, if any.
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

impl<R: Ord> LayoutRegistry<R> for Layouts<R> {
	fn get(&self, id: &Ref<LayoutType, R>) -> Option<&Layout<R>> {
		self.get(id)
	}
}

/// Layout definitions iterator.
///
/// Returned by the [`Layouts::iter`] method.
pub struct LayoutsIter<'a, R>(std::collections::btree_map::Iter<'a, R, Layout<R>>);

impl<'a, R> Iterator for LayoutsIter<'a, R> {
	type Item = (&'a Ref<LayoutType, R>, &'a Layout<R>);

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next().map(|(r, layout)| (Ref::new_ref(r), layout))
	}
}

impl<'a, R> IntoIterator for &'a Layouts<R> {
	type IntoIter = LayoutsIter<'a, R>;
	type Item = (&'a Ref<LayoutType, R>, &'a Layout<R>);

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

/// Layout definitions iterator.
///
/// Returned by the [`Layouts::into_iter`] method.
pub struct LayoutsIntoIter<R>(std::collections::btree_map::IntoIter<R, Layout<R>>);

impl<R> Iterator for LayoutsIntoIter<R> {
	type Item = (Ref<LayoutType, R>, Layout<R>);

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next().map(|(r, layout)| (Ref::new(r), layout))
	}
}

impl<R> IntoIterator for Layouts<R> {
	type IntoIter = LayoutsIntoIter<R>;
	type Item = (Ref<LayoutType, R>, Layout<R>);

	fn into_iter(self) -> Self::IntoIter {
		LayoutsIntoIter(self.layouts.into_iter())
	}
}

/// Union of two layout registries.
///
/// Registry `B` gets precedence over registry `A`. If a given layout is defined
/// in both registries, the definition provided by `B` will be returned.
pub struct LayoutRegistryUnion<A, B>(pub A, pub B);

impl<R, A: LayoutRegistry<R>, B: LayoutRegistry<R>> LayoutRegistry<R>
	for LayoutRegistryUnion<A, B>
{
	fn get(&self, id: &Ref<LayoutType, R>) -> Option<&Layout<R>> {
		self.1.get(id).or_else(|| self.0.get(id))
	}
}
