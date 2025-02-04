use std::{fmt, marker::PhantomData};

use btree_indexmap::BTreeIndexMap;

use crate::{value::Number, Literal, Value};

impl<'de, R: Ord + serde::Deserialize<'de>> serde::Deserialize<'de> for Value<R> {
	#[inline]
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		struct ValueVisitor<R>(PhantomData<R>);

		impl<'de, R: Ord + serde::Deserialize<'de>> serde::de::Visitor<'de> for ValueVisitor<R> {
			type Value = Value<R>;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				formatter.write_str("any valid TreeLDR value")
			}

			#[inline]
			fn visit_bool<E>(self, value: bool) -> Result<Value<R>, E> {
				Ok(Value::Literal(Literal::Boolean(value)))
			}

			#[inline]
			fn visit_i64<E>(self, value: i64) -> Result<Value<R>, E> {
				Ok(Value::Literal(Literal::Number(value.into())))
			}

			#[inline]
			fn visit_u64<E>(self, value: u64) -> Result<Value<R>, E> {
				Ok(Value::Literal(Literal::Number(value.into())))
			}

			#[inline]
			fn visit_f64<E>(self, value: f64) -> Result<Value<R>, E>
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
			fn visit_str<E>(self, value: &str) -> Result<Value<R>, E>
			where
				E: serde::de::Error,
			{
				self.visit_string(String::from(value))
			}

			#[inline]
			fn visit_string<E>(self, value: String) -> Result<Value<R>, E> {
				Ok(Value::Literal(Literal::TextString(value)))
			}

			#[inline]
			fn visit_none<E>(self) -> Result<Value<R>, E> {
				Ok(Value::Literal(Literal::Unit))
			}

			#[inline]
			fn visit_some<D>(self, deserializer: D) -> Result<Value<R>, D::Error>
			where
				D: serde::Deserializer<'de>,
			{
				serde::Deserialize::deserialize(deserializer)
			}

			#[inline]
			fn visit_unit<E>(self) -> Result<Value<R>, E> {
				Ok(Value::Literal(Literal::Unit))
			}

			#[inline]
			fn visit_seq<V>(self, mut visitor: V) -> Result<Value<R>, V::Error>
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
			fn visit_map<V>(self, mut visitor: V) -> Result<Value<R>, V::Error>
			where
				V: serde::de::MapAccess<'de>,
			{
				let mut map = BTreeIndexMap::new();

				while let Some((key, value)) = visitor.next_entry()? {
					map.insert(key, value);
				}

				Ok(Value::Map(map))
			}
		}

		deserializer.deserialize_any(ValueVisitor(PhantomData))
	}
}
