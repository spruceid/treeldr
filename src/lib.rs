use std::marker::PhantomData;

mod error;
pub mod source;
pub mod vocab;
pub mod ty;
pub mod syntax;

pub use error::Error;
pub use source::Source;
pub use vocab::Id;

/// Reference to an element of the context.
pub struct Ref<T>(usize, PhantomData<T>);

/// TreeLDR context.
pub struct Context {
	// ...
}