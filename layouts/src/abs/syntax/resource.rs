use json_syntax::{Kind, TryFromJsonSyntax};
use rdf_types::LexicalLiteralTypeRef;
use serde::{Deserialize, Serialize};
use xsd_types::{XSD_BOOLEAN, XSD_STRING};

use super::{require_entry, Build, BuildError, CompactIri, Context, Error, Scope};

/// RDF Resource description.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Resource {
	/// Boolean value.
	Boolean(bool),

	/// Decimal number value.
	Number(i64),

	/// Simple string literal.
	String(String),

	/// Typed string.
	TypedString(TypedString),
}

impl TryFromJsonSyntax for Resource {
	type Error = Error;

	fn try_from_json_syntax_at(
		json: &json_syntax::Value,
		code_map: &json_syntax::CodeMap,
		offset: usize,
	) -> Result<Self, Self::Error> {
		match json {
			json_syntax::Value::Boolean(b) => Ok(Self::Boolean(*b)),
			json_syntax::Value::Number(n) => {
				if n.contains('.') {
					Err(Error::ExpectedInteger(offset, n.clone()))
				} else {
					Ok(Self::Number(
						n.parse()
							.map_err(|_| Error::IntegerOverflow(offset, n.clone()))?,
					))
				}
			}
			json_syntax::Value::String(s) => Ok(Self::String(s.to_string())),
			json_syntax::Value::Object(object) => {
				// Typed string.
				Ok(Self::TypedString(TypedString::try_from_json_object_at(
					object, code_map, offset,
				)?))
			}
			other => Err(Error::Unexpected {
				offset,
				expected: Kind::Boolean | Kind::Number | Kind::String | Kind::Object,
				found: other.kind(),
			}),
		}
	}
}

impl<C: Context> Build<C> for Resource {
	type Target = C::Resource;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, BuildError> {
		match self {
			&Self::Boolean(b) => {
				let value = if b { "true" } else { "false" };

				Ok(context.literal_resource(value, LexicalLiteralTypeRef::Any(XSD_BOOLEAN)))
			}
			&Self::Number(n) => {
				let value: xsd_types::Decimal = n.into();
				let type_ = value.decimal_type();

				Ok(context.literal_resource(
					value.lexical_representation(),
					LexicalLiteralTypeRef::Any(type_.iri()),
				))
			}
			Self::String(value) => {
				Ok(context.literal_resource(value, LexicalLiteralTypeRef::Any(XSD_STRING)))
			}
			Self::TypedString(t) => t.build(context, scope),
		}
	}
}

/// Typed string literal.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TypedString {
	/// Literal value.
	pub value: String,

	/// Literal type.
	#[serde(rename = "type")]
	pub type_: CompactIri,
}

impl TypedString {
	fn try_from_json_object_at(
		object: &json_syntax::Object,
		code_map: &json_syntax::CodeMap,
		offset: usize,
	) -> Result<Self, Error> {
		Ok(Self {
			value: require_entry(object, "value", code_map, offset)?,
			type_: require_entry(object, "type", code_map, offset)?,
		})
	}
}

impl<C: Context> Build<C> for TypedString {
	type Target = C::Resource;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, BuildError> {
		let type_ = self.type_.resolve(scope)?;
		Ok(context.literal_resource(&self.value, LexicalLiteralTypeRef::Any(&type_)))
	}
}
