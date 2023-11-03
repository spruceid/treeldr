pub mod layout;

use std::marker::PhantomData;

use inferdf::Id;
pub use layout::Layout;

/// Typed RDF resource identifier.
pub struct Ref<T>(Id, PhantomData<T>);