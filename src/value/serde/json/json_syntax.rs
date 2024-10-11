impl TryFromJson for Value {
	type Error = std::convert::Infallible;

	fn try_from_json_at(
		json: &json_syntax::Value,
		code_map: &json_syntax::CodeMap,
		offset: usize,
	) -> Result<Self, Self::Error> {
		match json {
			json_syntax::Value::Null => Ok(Self::Literal(Literal::Unit)),
			json_syntax::Value::Boolean(b) => Ok(Self::Literal(Literal::Boolean(*b))),
			json_syntax::Value::Number(n) => {
				Ok(Self::Literal(Literal::Number(n.as_number().into())))
			}
			json_syntax::Value::String(s) => Ok(Self::Literal(Literal::TextString(s.to_string()))),
			json_syntax::Value::Array(a) => Ok(Self::List(
				a.iter_mapped(code_map, offset)
					.map(|item| Self::try_from_json_at(item.value, code_map, item.offset).unwrap())
					.collect(),
			)),
			json_syntax::Value::Object(o) => Ok(Self::Map(
				o.iter_mapped(code_map, offset)
					.map(|entry| {
						(
							Value::string(entry.value.key.value.to_string()),
							Self::try_from_json_at(
								entry.value.value.value,
								code_map,
								entry.value.value.offset,
							)
							.unwrap(),
						)
					})
					.collect(),
			)),
		}
	}
}

impl From<json_syntax::Value> for Value {
	fn from(value: json_syntax::Value) -> Self {
		match value {
			json_syntax::Value::Null => Self::Literal(Literal::Unit),
			json_syntax::Value::Boolean(b) => Self::Literal(Literal::Boolean(b)),
			json_syntax::Value::Number(n) => Self::Literal(Literal::Number(n.into())),
			json_syntax::Value::String(s) => Self::Literal(Literal::TextString(s.to_string())),
			json_syntax::Value::Array(a) => Self::List(a.into_iter().map(Into::into).collect()),
			json_syntax::Value::Object(o) => Self::Map(
				o.into_iter()
					.map(|entry| (Value::string(entry.key.into_string()), entry.value.into()))
					.collect(),
			),
		}
	}
}

impl TryFrom<Literal> for json_syntax::Value {
	type Error = NonJsonValue;

	fn try_from(value: Literal) -> Result<Self, Self::Error> {
		match value {
			Literal::Unit => Ok(json_syntax::Value::Null),
			Literal::Boolean(b) => Ok(json_syntax::Value::Boolean(b)),
			Literal::Number(n) => Ok(json_syntax::Value::Number(n.try_into()?)),
			Literal::TextString(s) => Ok(json_syntax::Value::String(s.into())),
			Literal::ByteString(s) => Err(NonJsonValue::ByteString(s)),
		}
	}
}

impl TryFrom<Value> for json_syntax::Value {
	type Error = NonJsonValue;

	fn try_from(value: Value) -> Result<Self, Self::Error> {
		match value {
			Value::Literal(l) => l.try_into(),
			Value::Map(r) => {
				let mut object = json_syntax::Object::new();

				for (key, value) in r {
					match key {
						Value::Literal(Literal::TextString(key)) => {
							object.insert(key.into(), value.try_into()?);
						}
						other => return Err(NonJsonValue::NonStringKey(other)),
					}
				}

				Ok(json_syntax::Value::Object(object))
			}
			Value::List(list) => list
				.into_iter()
				.map(TryInto::try_into)
				.collect::<Result<Vec<_>, _>>()
				.map(json_syntax::Value::Array),
		}
	}
}