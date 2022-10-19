pub use static_iref;

pub mod id;
pub mod rdf;

#[cfg(feature = "json-ld")]
pub mod json_ld;

pub use id::Id;
pub use rdf::{FromRdf, FromRdfError};

#[cfg(feature = "json-ld")]
pub use crate::json_ld::IntoJsonLd;
