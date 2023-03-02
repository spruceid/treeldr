pub use static_iref;
pub use rdf_types;
pub use grdf;

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