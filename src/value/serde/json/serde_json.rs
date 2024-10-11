impl From<serde_json::Value> for Value {
	fn from(value: serde_json::Value) -> Self {
		match value {
			serde_json::Value::Null => Self::Literal(Literal::Unit),
			serde_json::Value::Bool(b) => Self::Literal(Literal::Boolean(b)),
			serde_json::Value::Number(n) => Self::Literal(Literal::Number(n.into())),
			serde_json::Value::String(s) => Self::Literal(Literal::TextString(s)),
			serde_json::Value::Array(items) => {
				Self::List(items.into_iter().map(Into::into).collect())
			}
			serde_json::Value::Object(entries) => Self::Map(
				entries
					.into_iter()
					.map(|(key, value)| (Value::string(key), value.into()))
					.collect(),
			),
		}
	}
}

impl TryFrom<Number> for serde_json::Number {
	type Error = NonJsonNumber;

	fn try_from(value: Number) -> Result<Self, Self::Error> {
		match value.decimal_representation() {
			Some(decimal) => match serde_json::Number::from_str(&decimal) {
				Ok(n) => Ok(n),
				Err(_) => Err(NonJsonNumber(value)),
			},
			None => Err(NonJsonNumber(value)),
		}
	}
}

impl TryFrom<Literal> for serde_json::Value {
	type Error = NonJsonValue;

	fn try_from(value: Literal) -> Result<Self, Self::Error> {
		match value {
			Literal::Unit => Ok(serde_json::Value::Null),
			Literal::Boolean(b) => Ok(serde_json::Value::Bool(b)),
			Literal::Number(n) => Ok(serde_json::Value::Number(n.try_into()?)),
			Literal::TextString(s) => Ok(serde_json::Value::String(s)),
			Literal::ByteString(s) => Err(NonJsonValue::ByteString(s)),
		}
	}
}

impl TryFrom<TypedLiteral> for serde_json::Value {
	type Error = NonJsonValue;

	fn try_from(value: TypedLiteral) -> Result<Self, Self::Error> {
		match value {
			TypedLiteral::Id(s, _) => Ok(serde_json::Value::String(s)),
			TypedLiteral::Unit(value, _) => value.try_into(),
			TypedLiteral::Boolean(b, _) => Ok(serde_json::Value::Bool(b)),
			TypedLiteral::Number(n, _) => Ok(serde_json::Value::Number(n.try_into()?)),
			TypedLiteral::TextString(s, _) => Ok(serde_json::Value::String(s)),
			TypedLiteral::ByteString(s, _) => Err(NonJsonValue::ByteString(s)),
		}
	}
}

impl TryFrom<Value> for serde_json::Value {
	type Error = NonJsonValue;

	fn try_from(value: Value) -> Result<Self, Self::Error> {
		match value {
			Value::Literal(l) => l.try_into(),
			Value::Map(r) => {
				let mut object = serde_json::Map::new();

				for (key, value) in r {
					match key {
						Value::Literal(Literal::TextString(key)) => {
							object.insert(key, value.try_into()?);
						}
						other => return Err(NonJsonValue::NonStringKey(other)),
					}
				}

				Ok(serde_json::Value::Object(object))
			}
			Value::List(list) => list
				.into_iter()
				.map(TryInto::try_into)
				.collect::<Result<Vec<_>, _>>()
				.map(serde_json::Value::Array),
		}
	}
}