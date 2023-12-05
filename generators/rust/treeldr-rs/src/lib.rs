use iref::Iri;
use rdf_types::BlankId;
#[cfg(feature = "derive")]
pub use treeldr_derive::{DeserializeLd, SerializeLd};

#[doc(hidden)]
pub use iref;

#[doc(hidden)]
pub use rdf_types;

#[doc(hidden)]
pub use grdf;

mod datatypes;
pub mod de;
pub mod pattern;
mod rdf;
pub mod ser;
pub mod utils;

pub use de::{DeserializeLd, Error as DeserializeError};
pub use pattern::Pattern;
pub use rdf::{RdfContext, RdfContextMut, RdfType};
pub use ser::{Error as SerializeError, SerializeLd};

pub trait AsId {
	fn as_id(&self) -> rdf_types::Id<&Iri, &BlankId>;
}

impl AsId for rdf_types::Id {
	fn as_id(&self) -> rdf_types::Id<&Iri, &BlankId> {
		self.as_id_ref()
	}
}
