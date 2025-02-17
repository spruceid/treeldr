//! This library can parse, build and evaluate programs written with TreeLDR a
//! functional data transformation language based on the Resource Description
//! Framework (RDF). It can be used to serialize RDF datasets into algebraic
//! data values, and back. What makes TreeLDR unique is that functions are
//! **always reversible**: writing a serialization function gives the
//! deserialization function for free.
//!
//! # Usage
//!
//! ```
//! use treeldr::{
//!   registry::{Registry, RegistryMut, PreludeAndTarget, PreludeOrTarget},
//!   rdf::{dataset::BTreeDataset, generator, parse_nquads}
//! };
//! use serde_json::json;
//!
//! // Create a basic registry with a prelude and a target package.
//! // The prelude package provides built-in types and function.
//! // The target package is not set yet, we are going to compile it.
//! let mut registry = PreludeAndTarget::new();
//!
//! // Compile a TreeLDR package, and insert it in the registry (as our
//! // "target" package).
//! registry.compile_and_insert_package(
//!   PreludeOrTarget::Target,
//!   r#"
//!     prefix ex = <http://example.com/>
//!
//!     let (function_name input) = {
//!       "id": (iri_of input),
//!       "foo": (string_of (get ex:foo input)),
//!       "bar": (iri_of (get ex:bar input)),
//!     }
//!   "#
//! ).unwrap();
//!
//! // Retrieve a function from the target package we just compiled.
//! let function = registry
//!   .package(PreludeOrTarget::Target)
//!   .unwrap()
//!   .root()
//!   .get_by_name("function_name")
//!   .unwrap()
//!   .as_function()
//!   .unwrap()
//!   .get(&registry)
//!   .unwrap();
//!
//! // Create a new RDF dataset.
//! let mut dataset = parse_nquads(r#"
//!   <http://example.com/subject> <http://example.com/foo> "Value" .
//!   <http://example.com/subject> <http://example.com/bar> <http://example.com/object> .
//! "#).unwrap().into_indexed();
//!
//! let mut generator = generator::Blank::new();
//!
//! // Call the function.
//! let output = function.call(
//!   &registry,
//!   &mut generator,
//!   &mut dataset,
//!   &["<http://example.com/subject>".parse().unwrap()]
//! ).unwrap();
//!
//! assert_eq!(
//!   output,
//!   serde_json::from_value(json!({
//!     "id": "http://example.com/subject",
//!     "foo": "Value",
//!     "bar": "http://example.com/object"
//!   })).unwrap()
//! );
//!
//! // Call the function's inverse.
//! let inputs = function.call_inverse(
//!   &registry,
//!   &mut generator,
//!   &mut dataset,
//!   &[],
//!   &output
//! ).unwrap();
//!
//! assert_eq!(
//!   inputs,
//!   vec!["<http://example.com/subject>".parse().unwrap()]
//! );
//! ```
pub mod attributes;
pub mod build;
pub mod display;
pub mod eval;
pub mod expr;
pub mod function;
pub mod matching;
pub mod module;
pub mod package;
pub mod pattern;
mod prelude;
pub mod rdf;
pub mod registry;
pub mod syntax;
pub mod ty;
pub mod utils;
pub mod value;

pub use attributes::{Attribute, Attributes};
pub use function::Function;
pub use matching::Matching;
use module::{ItemPath, ItemPathSegment, ItemRef};
pub use module::{Module, ModuleRef};
pub use package::Package;
pub use pattern::{DatasetPattern, Graph, Pattern, TermPattern};
pub use registry::*;
pub use ty::{Type, TypeRef};
pub use value::{Literal, Value};
