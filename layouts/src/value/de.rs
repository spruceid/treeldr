use std::{collections::BTreeMap, fmt};

use super::{Literal, Number, Value};

impl<'de> serde::Deserialize<'de> for Value {
	#[inline]
	fn deserialize<D>(deserializer: D) -> Result<Value, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		struct ValueVisitor;

		impl<'de> serde::de::Visitor<'de> for ValueVisitor {
			type Value = Value;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				formatter.write_str("any valid TreeLDR value")
			}

			#[inline]
			fn visit_bool<E>(self, value: bool) -> Result<Value, E> {
				Ok(Value::Literal(Literal::Boolean(value)))
			}

			#[inline]
			fn visit_i64<E>(self, value: i64) -> Result<Value, E> {
				Ok(Value::Literal(Literal::Number(value.into())))
			}

			#[inline]
			fn visit_u64<E>(self, value: u64) -> Result<Value, E> {
				Ok(Value::Literal(Literal::Number(value.into())))
			}

			#[inline]
			fn visit_f64<E>(self, value: f64) -> Result<Value, E>
			where
				E: serde::de::Error,
			{
				match Number::try_from(value) {
					Ok(value) => Ok(Value::Literal(Literal::Number(value))),
					Err(_) => Err(E::invalid_value(serde::de::Unexpected::Float(value), &self)),
				}
			}

			fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				self.visit_byte_buf(value.to_vec())
			}

			fn visit_byte_buf<E>(self, value: Vec<u8>) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				Ok(Value::Literal(Literal::ByteString(value)))
			}

			#[inline]
			fn visit_str<E>(self, value: &str) -> Result<Value, E>
			where
				E: serde::de::Error,
			{
				self.visit_string(String::from(value))
			}

			#[inline]
			fn visit_string<E>(self, value: String) -> Result<Value, E> {
				Ok(Value::Literal(Literal::TextString(value)))
			}

			#[inline]
			fn visit_none<E>(self) -> Result<Value, E> {
				Ok(Value::Literal(Literal::Unit))
			}

			#[inline]
			fn visit_some<D>(self, deserializer: D) -> Result<Value, D::Error>
			where
				D: serde::Deserializer<'de>,
			{
				serde::Deserialize::deserialize(deserializer)
			}

			#[inline]
			fn visit_unit<E>(self) -> Result<Value, E> {
				Ok(Value::Literal(Literal::Unit))
			}

			#[inline]
			fn visit_seq<V>(self, mut visitor: V) -> Result<Value, V::Error>
			where
				V: serde::de::SeqAccess<'de>,
			{
				let mut vec = Vec::new();

				while let Some(elem) = visitor.next_element()? {
					vec.push(elem);
				}

				Ok(Value::List(vec))
			}

			#[inline]
			fn visit_map<V>(self, mut visitor: V) -> Result<Value, V::Error>
			where
				V: serde::de::MapAccess<'de>,
			{
				let mut map = BTreeMap::new();

				while let Some((key, value)) = visitor.next_entry()? {
					map.insert(key, value);
				}

				Ok(Value::Record(map))
			}
		}

		deserializer.deserialize_any(ValueVisitor)
	}
}
