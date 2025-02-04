use core::fmt;
pub mod number;
pub use number::{NativeNumber, Number};
use serde_seeded::{DeserializeSeeded, SerializeSeeded};

use crate::display::{string_literal, StringLiteralDelimiter};

/// Literal value.
#[derive(
	Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SerializeSeeded, DeserializeSeeded,
)]
#[seeded(ser(params(Q), seed(Q)), de(params(Q), seed(Q)))]
pub enum Literal {
	/// Unit.
	Unit,

	/// Boolean value.
	Boolean(bool),

	/// Any rational number.
	Number(Number),

	/// Byte string.
	ByteString(#[seeded(with(serde_bytes))] Vec<u8>),

	/// Text string.
	TextString(String),
}

impl Literal {
	pub fn as_bool(&self) -> Option<bool> {
		match self {
			Self::Boolean(b) => Some(*b),
			_ => None,
		}
	}

	pub fn as_number(&self) -> Option<&Number> {
		match self {
			Self::Number(n) => Some(n),
			_ => None,
		}
	}

	pub fn as_str(&self) -> Option<&str> {
		match self {
			Self::TextString(s) => Some(s),
			_ => None,
		}
	}
}

impl fmt::Display for Literal {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Unit => f.write_str("null"),
			Self::Boolean(true) => f.write_str("true"),
			Self::Boolean(false) => f.write_str("false"),
			Self::Number(n) => n.fmt(f),
			Self::ByteString(bytes) => {
				f.write_str("x")?;
				for b in bytes {
					write!(f, "{b:02x}")?;
				}
				Ok(())
			}
			Self::TextString(s) => string_literal(s, StringLiteralDelimiter::Double, f),
		}
	}
}

mod serde_bytes {
	use serde::{Deserializer, Serializer};

	pub fn serialize_seeded<Q, S>(
		bytes: &[u8],
		_index: &Q,
		serializer: S,
	) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_bytes(bytes)
	}

	pub fn deserialize_seeded<'de, Q, D>(_index: &Q, deserializer: D) -> Result<Vec<u8>, D::Error>
	where
		D: Deserializer<'de>,
	{
		struct Visitor;

		impl<'de> serde::de::Visitor<'de> for Visitor {
			type Value = Vec<u8>;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				write!(formatter, "a byte string")
			}

			fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				Ok(v.to_owned())
			}

			fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				Ok(v)
			}
		}

		deserializer.deserialize_bytes(Visitor)
	}
}
