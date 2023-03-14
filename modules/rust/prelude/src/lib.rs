pub use contextual;
pub use grdf;
pub use rdf_types;
pub use static_iref;

pub mod iter;

#[cfg(feature = "json-ld")]
pub use json_ld;

#[cfg(feature = "json-ld")]
pub use locspan;

pub mod rdf;

#[cfg(feature = "json-ld")]
pub mod ld;

pub use rdf::{FromRdf, FromRdfError, RdfIterator};

#[cfg(feature = "json-ld")]
pub use crate::ld::IntoJsonLd;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id<I>(pub I);

impl<I> Id<I> {
	pub fn unwrap(self) -> I {
		self.0
	}
}

impl<I> From<I> for Id<I> {
	fn from(value: I) -> Self {
		Id(value)
	}
}

pub trait Provider<I: ?Sized, T: ?Sized> {
	fn get(&self, id: &I) -> Option<&T>;
}
