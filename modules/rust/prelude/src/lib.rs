pub use chrono;
pub use contextual;
pub use grdf;
pub use iref;
pub use rdf_types;
pub use static_iref;

pub mod iter;

#[cfg(feature = "json-ld")]
pub use json_ld;

#[cfg(feature = "json-ld")]
pub use locspan;

pub mod restriction;
pub mod ty;

pub mod rdf;
pub use rdf::{FromRdf, FromRdfError, FromRdfLiteral, RdfIterator};

#[cfg(feature = "json-ld")]
pub mod ld;
#[cfg(feature = "json-ld")]
pub use crate::ld::{
	AsJsonLd, AsJsonLdObject, AsJsonLdObjectMeta, IntoJsonLd, IntoJsonLdObject,
	IntoJsonLdObjectMeta, IntoJsonLdSyntax,
};

pub enum Ref<'r, I: ?Sized, T: ?Sized> {
	Id(&'r I),
	Value(&'r T),
}

pub trait Provider<Id: ?Sized, T: ?Sized> {
	fn get(&self, id: &Id) -> Option<&T>;
}

/// Resource identifier wrapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id<I>(pub I);

impl<I> Id<I> {
	pub fn unwrap(self) -> I {
		self.0
	}
}
