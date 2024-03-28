use rdf_types::LexicalLiteralTypeRef;
use serde::{Deserialize, Serialize};
use xsd_types::{XSD_BOOLEAN, XSD_STRING};

use super::{Build, CompactIri, Context, BuildError, Scope};

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

impl<C: Context> Build<C> for TypedString {
	type Target = C::Resource;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, BuildError> {
		let type_ = self.type_.resolve(scope)?;
		Ok(context.literal_resource(&self.value, LexicalLiteralTypeRef::Any(&type_)))
	}
}
