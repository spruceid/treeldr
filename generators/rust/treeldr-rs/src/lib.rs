use iref::Iri;
use rdf_types::BlankId;

#[cfg(feature = "macros")]
/// Embed TreeLDR layouts as Rust types in the given module.
///
/// # Example
///
/// ```
/// use treeldr::tldr;
/// #[tldr("layouts/examples/simple_record.json")]
/// mod module {
///   // a `SimpleLayout` type will be generated here.
/// }
/// ```
pub use treeldr_macros::tldr;

#[cfg(feature = "macros")]
/// Embed TreeLDR layouts as Rust types.
///
/// # Example
///
/// ```
/// # use treeldr_macros::tldr_include;
/// tldr_include!("layouts/examples/simple_record.json");
/// ```
pub use treeldr_macros::tldr_include;

#[cfg(feature = "macros")]
pub use treeldr_macros::{DeserializeLd, SerializeLd};

#[doc(hidden)]
pub use iref;

#[doc(hidden)]
pub use rdf_types;

mod datatypes;
pub mod de;
pub mod pattern;
mod rdf;
pub mod ser;
pub mod utils;

pub use de::{DeserializeLd, Error as DeserializeError};
pub use pattern::Pattern;
pub use rdf::{RdfContext, RdfContextMut};
pub use ser::{Error as SerializeError, SerializeLd};

pub trait AsId {
	fn as_id(&self) -> rdf_types::Id<&Iri, &BlankId>;
}

impl AsId for rdf_types::Id {
	fn as_id(&self) -> rdf_types::Id<&Iri, &BlankId> {
		self.as_lexical_id_ref()
	}
}
