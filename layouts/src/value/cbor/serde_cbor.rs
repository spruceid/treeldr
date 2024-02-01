use std::borrow::Borrow;

use num_traits::{Signed, ToBytes};
use rdf_types::{Interpretation, IriInterpretation, ReverseLiteralInterpretation, Vocabulary};

use crate::{value::Number, Layouts, Literal, TypedLiteral, TypedValue, Value};

use super::{get_layout_tag, InvalidTag};

impl TypedValue {
	/// Tries to convert this TreeLDR value into a CBOR value, using CBOR tags
	/// as specified by the layout property
	/// <https://schema.treeldr.org/cbor#tag>.
	pub fn try_into_tagged_serde_cbor(
		self,
		layouts: &Layouts,
	) -> Result<serde_cbor::Value, InvalidTag> {
		self.try_into_tagged_serde_cbor_with(&(), &(), layouts)
	}
}

impl<R> TypedValue<R> {
	/// Tries to convert this TreeLDR value into a CBOR value, using CBOR tags
	/// as specified by the layout property
	/// <https://schema.treeldr.org/cbor#tag>.
	pub fn try_into_tagged_serde_cbor_with<V, I>(
		self,
		vocabulary: &V,
		interpretation: &I,
		layouts: &Layouts<R>,
	) -> Result<serde_cbor::Value, InvalidTag>
	where
		V: Vocabulary,
		V::Value: AsRef<str>,
		V::Type: Borrow<rdf_types::literal::Type<V::Iri, V::LanguageTag>>,
		I: Interpretation<Resource = R>
			+ IriInterpretation<V::Iri>
			+ ReverseLiteralInterpretation<Literal = V::Literal>,
		R: Ord,
	{
		let (value, ty) = match self {
			Self::Always(value) => (value.into(), None),
			Self::Literal(TypedLiteral::Unit(_, ty)) => (serde_cbor::Value::Null, Some(ty.cast())),
			Self::Literal(TypedLiteral::Boolean(b, ty)) => {
				(serde_cbor::Value::Bool(b), Some(ty.cast()))
			}
			Self::Literal(TypedLiteral::Number(n, ty)) => (n.into(), Some(ty.cast())),
			Self::Literal(TypedLiteral::ByteString(b, ty)) => {
				(serde_cbor::Value::Bytes(b), Some(ty.cast()))
			}
			Self::Literal(TypedLiteral::TextString(s, ty)) => {
				(serde_cbor::Value::Text(s), Some(ty.cast()))
			}
			Self::Literal(TypedLiteral::Id(s, ty)) => (serde_cbor::Value::Text(s), Some(ty.cast())),
			Self::Variant(inner, ty, _) => (
				inner.try_into_tagged_serde_cbor_with(vocabulary, interpretation, layouts)?,
				Some(ty.cast()),
			),
			Self::Record(map, ty) => (
				serde_cbor::Value::Map(
					map.into_iter()
						.map(|(key, value)| {
							Ok((
								serde_cbor::Value::Text(key),
								value.try_into_tagged_serde_cbor_with(
									vocabulary,
									interpretation,
									layouts,
								)?,
							))
						})
						.collect::<Result<_, _>>()?,
				),
				Some(ty.cast()),
			),
			Self::List(items, ty) => (
				serde_cbor::Value::Array(
					items
						.into_iter()
						.map(|t| {
							t.try_into_tagged_serde_cbor_with(vocabulary, interpretation, layouts)
						})
						.collect::<Result<_, _>>()?,
				),
				Some(ty.cast()),
			),
		};

		if let Some(ty) = ty {
			if let Some(tag) = get_layout_tag(vocabulary, interpretation, layouts, &ty)? {
				return Ok(serde_cbor::Value::Tag(tag, Box::new(value)));
			}
		}

		Ok(value)
	}
}

impl<R> From<TypedValue<R>> for serde_cbor::Value {
	fn from(value: TypedValue<R>) -> Self {
		match value {
			TypedValue::Always(value) => value.into(),
			TypedValue::Literal(TypedLiteral::Unit(_, _)) => Self::Null,
			TypedValue::Literal(TypedLiteral::Boolean(b, _)) => Self::Bool(b),
			TypedValue::Literal(TypedLiteral::Number(n, _)) => n.into(),
			TypedValue::Literal(TypedLiteral::ByteString(b, _)) => Self::Bytes(b),
			TypedValue::Literal(TypedLiteral::TextString(s, _)) => Self::Text(s),
			TypedValue::Literal(TypedLiteral::Id(s, _)) => Self::Text(s),
			TypedValue::Variant(inner, _, _) => (*inner).into(),
			TypedValue::Record(map, _) => Self::Map(
				map.into_iter()
					.map(|(key, value)| (serde_cbor::Value::Text(key), value.into()))
					.collect(),
			),
			TypedValue::List(items, _) => Self::Array(items.into_iter().map(Into::into).collect()),
		}
	}
}

impl From<Number> for serde_cbor::Value {
	fn from(value: Number) -> Self {
		match value.as_integer() {
			Some(i) => {
				if i.bits() <= 64 {
					let unsigned = i.iter_u64_digits().next().unwrap() as i128;

					let signed = if i.is_positive() { unsigned } else { -unsigned };

					serde_cbor::Value::Integer(signed)
				} else {
					let tag = if i.is_positive() { 2 } else { 3 };

					serde_cbor::Value::Tag(tag, Box::new(serde_cbor::Value::Bytes(i.to_be_bytes())))
				}
			}
			None => serde_cbor::Value::Float(value.to_f64()),
		}
	}
}

impl From<Value> for serde_cbor::Value {
	fn from(value: Value) -> Self {
		match value {
			Value::Literal(Literal::Unit) => serde_cbor::Value::Null,
			Value::Literal(Literal::Boolean(b)) => serde_cbor::Value::Bool(b),
			Value::Literal(Literal::Number(n)) => n.into(),
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

#[cfg(test)]
mod tests {
	use crate::{
		abs::{self, syntax},
		hydrate,
	};
	use grdf::BTreeDataset;
	use rdf_types::{literal::Type, BlankIdBuf, Literal, Quad, Term};
	use serde_json::json;
	use static_iref::iri;
	use xsd_types::XSD_STRING;

	#[test]
	fn into_tagged_cbor() {
		let layout: syntax::Layout = serde_json::from_value(json!(
			{
				"prefixes": {
					"cbor": "https://schema.treeldr.org/cbor#"
				},
				"type": "record",
				"fields": {
					"name": {
						"value": {
							"type": "string",
							"extra": {
								"cbor:tag": 42
							}
						},
						"property": "https://schema.org/name"
					}
				}
			}
		))
		.unwrap();

		let mut builder = abs::Builder::new();
		let layout_ref = layout.build(&mut builder).unwrap();
		let layouts = builder.build();

		let mut dataset = BTreeDataset::new();
		let subject = Term::blank(BlankIdBuf::from_suffix("subject").unwrap());
		dataset.insert(Quad(
			subject.clone(),
			Term::iri(iri!("https://schema.org/name").to_owned()),
			Term::Literal(Literal::new(
				"Bob L'éponge".to_owned(),
				Type::Any(XSD_STRING.to_owned()),
			)),
			None,
		));

		let value = hydrate(&layouts, &dataset, &layout_ref, &[subject]).unwrap();
		let output = value.try_into_tagged_serde_cbor(&layouts).unwrap();

		let expected = serde_cbor::Value::Map(
			[(
				"name".to_owned().into(),
				serde_cbor::Value::Tag(42, Box::new("Bob L'éponge".to_owned().into())),
			)]
			.into_iter()
			.collect(),
		);

		assert_eq!(output, expected)
	}
}
