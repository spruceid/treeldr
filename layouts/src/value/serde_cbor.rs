use num_traits::{Signed, ToBytes};

use super::{Literal, Value};

impl From<Value> for serde_cbor::Value {
	fn from(value: Value) -> Self {
		match value {
			Value::Literal(Literal::Unit) => serde_cbor::Value::Null,
			Value::Literal(Literal::Boolean(b)) => serde_cbor::Value::Bool(b),
			Value::Literal(Literal::Number(n)) => match n.as_integer() {
				Some(i) => {
					if i.bits() <= 64 {
						let unsigned = i.iter_u64_digits().next().unwrap() as i128;

						let signed = if i.is_positive() { unsigned } else { -unsigned };

						serde_cbor::Value::Integer(signed)
					} else {
						let tag = if i.is_positive() { 2 } else { 3 };

						serde_cbor::Value::Tag(
							tag,
							Box::new(serde_cbor::Value::Bytes(i.to_be_bytes())),
						)
					}
				}
				None => serde_cbor::Value::Float(n.to_f64()),
			},
			Value::Literal(Literal::ByteString(bytes)) => serde_cbor::Value::Bytes(bytes),
			Value::Literal(Literal::TextString(string)) => serde_cbor::Value::Text(string),
			Value::Record(map) => serde_cbor::Value::Map(
				map.into_iter()
					.map(|(key, value)| (serde_cbor::Value::Text(key), value.into()))
					.collect(),
			),
			Value::List(items) => {
				serde_cbor::Value::Array(items.into_iter().map(Into::into).collect())
			}
		}
	}
}
