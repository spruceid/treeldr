use crate::{value::Number, Value};

#[cfg(feature = "json-syntax")]
mod json_syntax;

#[cfg(feature = "serde_json")]
mod serde_json;

/// Error raised when trying to convert a value to JSON that is not compatible
/// with the JSON data model.
#[derive(Debug, thiserror::Error)]
pub enum NonJsonValue<R = rdf_types::Term> {
	/// RDF resource value, not supported by JSON.
	#[error("RDF resource cannot be converted to JSON")]
	Resource,

	/// Literal is not supported by JSON.
	#[error(transparent)]
	Literal(#[from] NonJsonLiteral),

	/// Non-string key, not supported by JSON.
	#[error("non-string key {0:?}")]
	NonStringKey(Value<R>),
}

impl<R> From<NonJsonNumber> for NonJsonValue<R> {
	fn from(value: NonJsonNumber) -> Self {
		NonJsonValue::Literal(value.into())
	}
}

/// Error raised when trying to convert a literal to JSON that is not compatible
/// with the JSON data model.
#[derive(Debug, thiserror::Error)]
pub enum NonJsonLiteral {
	/// Number cannot be represented as JSON.
	#[error("not a JSON number: {0}")]
	Number(Number),

	/// Byte string value, not supported by JSON.
	#[error("byte string cannot be converted to JSON")]
	ByteString(Vec<u8>),
}

impl From<NonJsonNumber> for NonJsonLiteral {
	fn from(value: NonJsonNumber) -> Self {
		Self::Number(value.0)
	}
}

/// Error raised when trying to convert a non-decimal number to JSON.
#[derive(Debug, thiserror::Error)]
#[error("not a JSON number: {0}")]
pub struct NonJsonNumber(pub Number);
