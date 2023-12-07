use super::{Literal, NativeNumber, Value};

impl serde::Serialize for Value {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		match self {
			Self::Literal(literal) => literal.serialize(serializer),
			Self::Record(record) => {
				use serde::ser::SerializeMap;
				let mut map = serializer.serialize_map(Some(record.len()))?;

				for (key, value) in record {
					map.serialize_entry(key, value)?;
				}

				map.end()
			}
			Self::List(list) => {
				use serde::ser::SerializeSeq;
				let mut seq = serializer.serialize_seq(Some(list.len()))?;

				for item in list {
					seq.serialize_element(item)?;
				}

				seq.end()
			}
		}
	}
}

impl serde::Serialize for Literal {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		match self {
			Self::Unit => serializer.serialize_unit(),
			Self::Boolean(b) => serializer.serialize_bool(*b),
			Self::Number(n) => match n.as_native() {
				NativeNumber::U64(u) => serializer.serialize_u64(u),
				NativeNumber::I64(i) => serializer.serialize_i64(i),
				NativeNumber::F64(f) => serializer.serialize_f64(f),
			},
			Self::TextString(s) => serializer.serialize_str(s),
			Self::ByteString(b) => serializer.serialize_bytes(b),
		}
	}
}
