/// Error raised when trying to convert a value to JSON that is not compatible
/// with the JSON data model.
#[derive(Debug, thiserror::Error)]
pub enum NonJsonValue {
	/// Number cannot be represented as JSON.
	#[error("not a JSON number: {0}")]
	Number(Number),

	/// Byte string value, not supported by JSON.
	#[error("byte string cannot be converted to JSON")]
	ByteString(Vec<u8>),

	/// Non-string key, not supported by JSON.
	#[error("non-string key")]
	NonStringKey(Value),
}

impl From<NonJsonNumber> for NonJsonValue {
	fn from(value: NonJsonNumber) -> Self {
		NonJsonValue::Number(value.0)
	}
}