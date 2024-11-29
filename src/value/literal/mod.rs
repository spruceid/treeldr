use core::fmt;
use serde::{Deserialize, Serialize};

pub mod number;
pub use number::{NativeNumber, Number};
use treeldr_macros::{DeserializeIndexed, SerializeIndexed};

use crate::rdf::serde::{DeserializeIndexed, SerializeIndexed};

/// Literal value.
#[derive(
	Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SerializeIndexed, DeserializeIndexed,
)]
pub enum Literal {
	/// Unit.
	Unit,

	/// Boolean value.
	Boolean(bool),

	/// Any rational number.
	Number(Number),

	/// Byte string.
	ByteString(#[serde_indexed(with = serde_bytes)] Vec<u8>),

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
			Self::TextString(s) => string_literal(s, f),
		}
	}
}

/// Formats a string literal according to [RFC8785](https://www.rfc-editor.org/rfc/rfc8785#name-serialization-of-strings).
pub fn string_literal(s: &str, f: &mut fmt::Formatter) -> fmt::Result {
	use fmt::Display;
	f.write_str("\"")?;

	for c in s.chars() {
		match c {
			'\\' => f.write_str("\\\\")?,
			'\"' => f.write_str("\\\"")?,
			'\u{0008}' => f.write_str("\\b")?,
			'\u{0009}' => f.write_str("\\t")?,
			'\u{000a}' => f.write_str("\\n")?,
			'\u{000c}' => f.write_str("\\f")?,
			'\u{000d}' => f.write_str("\\r")?,
			'\u{0000}'..='\u{001f}' => {
				f.write_str("\\u")?;

				let codepoint = c as u32;
				let d = codepoint & 0x000f;
				let c = (codepoint & 0x00f0) >> 4;
				let b = (codepoint & 0x0f00) >> 8;
				let a = (codepoint & 0xf000) >> 12;

				digit(a).fmt(f)?;
				digit(b).fmt(f)?;
				digit(c).fmt(f)?;
				digit(d).fmt(f)?
			}
			_ => c.fmt(f)?,
		}
	}

	f.write_str("\"")
}

fn digit(c: u32) -> char {
	match c {
		0x0 => '0',
		0x1 => '1',
		0x2 => '2',
		0x3 => '3',
		0x4 => '4',
		0x5 => '5',
		0x6 => '6',
		0x7 => '7',
		0x8 => '8',
		0x9 => '9',
		0xa => 'a',
		0xb => 'b',
		0xc => 'c',
		0xd => 'd',
		0xe => 'e',
		0xf => 'f',
		_ => panic!("invalid input: {}", c),
	}
}

impl<R> SerializeIndexed<R> for () {
	fn serialize_indexed<S>(
		&self,
		_index: &crate::rdf::ResourceForwardIndex<R>,
		serializer: S,
	) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.serialize(serializer)
	}
}

impl<'de, R> DeserializeIndexed<'de, R> for () {
	fn deserialize_indexed<D>(
		_index: &crate::rdf::ResourceBackwardIndex<R>,
		deserializer: D,
	) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		Self::deserialize(deserializer)
	}
}

impl<R> SerializeIndexed<R> for bool {
	fn serialize_indexed<S>(
		&self,
		_index: &crate::rdf::ResourceForwardIndex<R>,
		serializer: S,
	) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.serialize(serializer)
	}
}

impl<'de, R> DeserializeIndexed<'de, R> for bool {
	fn deserialize_indexed<D>(
		_index: &crate::rdf::ResourceBackwardIndex<R>,
		deserializer: D,
	) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		Self::deserialize(deserializer)
	}
}

impl<R> SerializeIndexed<R> for String {
	fn serialize_indexed<S>(
		&self,
		_index: &crate::rdf::ResourceForwardIndex<R>,
		serializer: S,
	) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.serialize(serializer)
	}
}

impl<'de, R> DeserializeIndexed<'de, R> for String {
	fn deserialize_indexed<D>(
		_index: &crate::rdf::ResourceBackwardIndex<R>,
		deserializer: D,
	) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		Self::deserialize(deserializer)
	}
}

impl<R> SerializeIndexed<R> for Vec<u8> {
	fn serialize_indexed<S>(
		&self,
		_index: &crate::rdf::ResourceForwardIndex<R>,
		_serializer: S,
	) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		todo!()
	}
}

mod serde_bytes {
	use crate::rdf::ResourceBackwardIndex;

	pub struct Seed;

	impl<'de> serde::de::DeserializeSeed<'de> for Seed {
		type Value = Vec<u8>;

		fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
		where
			D: serde::Deserializer<'de>,
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

	pub fn deserialize_seed<R>(_index: &ResourceBackwardIndex<R>) -> Seed {
		Seed
	}
}
