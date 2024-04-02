use iref::IriBuf;
use langtag::LangTagBuf;
use rdf_types::XSD_STRING;

use crate::abs::syntax::{get_entry, require_entry, BuildError, Error, ObjectUnusedEntries, Scope};

use super::CompactIri;

#[derive(
	Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct LiteralValue {
	pub value: String,

	#[serde(flatten)]
	pub type_: LiteralType,
}

impl LiteralValue {
	pub fn try_from_json_object_at(
		object: &json_syntax::Object,
		code_map: &json_syntax::CodeMap,
		offset: usize,
	) -> Result<Self, Error> {
		let mut unused_entries = ObjectUnusedEntries::new(object, code_map, offset);
		let type_ = match get_entry(object, "type", &mut unused_entries, code_map, offset)? {
			Some(ty) => {
				// TODO check if language is present.
				LiteralType::Iri(LiteralTypeIri { type_: ty })
			}
			None => {
				let language: String =
					require_entry(object, "language", &mut unused_entries, code_map, offset)?;
				match LangTagBuf::new(language) {
					Ok(language) => LiteralType::Language(LiteralTypeLanguage { language }),
					Err(e) => return Err(Error::InvalidLangTag(offset, e.0)),
				}
			}
		};

		let value = require_entry(object, "value", &mut unused_entries, code_map, offset)?;
		unused_entries.check()?;

		Ok(Self { value, type_ })
	}
}

impl From<rdf_types::Literal> for LiteralValue {
	fn from(value: rdf_types::Literal) -> Self {
		Self {
			value: value.value,
			type_: value.type_.into(),
		}
	}
}

#[derive(
	Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(untagged)]
pub enum LiteralType {
	Iri(LiteralTypeIri),
	Language(LiteralTypeLanguage),
}

impl Default for LiteralType {
	fn default() -> Self {
		Self::Iri(LiteralTypeIri {
			type_: XSD_STRING.to_owned().into(),
		})
	}
}

impl LiteralType {
	pub fn resolve(&self, scope: &Scope) -> Result<rdf_types::LiteralType, BuildError> {
		match self {
			Self::Iri(iri) => Ok(rdf_types::LiteralType::Any(iri.resolve(scope)?)),
			Self::Language(lang) => Ok(rdf_types::LiteralType::LangString(lang.language.clone())),
		}
	}
}

impl From<rdf_types::LiteralType> for LiteralType {
	fn from(value: rdf_types::LiteralType) -> Self {
		match value {
			rdf_types::LiteralType::Any(iri) => Self::Iri(LiteralTypeIri { type_: iri.into() }),
			rdf_types::LiteralType::LangString(tag) => {
				Self::Language(LiteralTypeLanguage { language: tag })
			}
		}
	}
}

#[derive(
	Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct LiteralTypeIri {
	#[serde(
		rename = "type",
		skip_serializing_if = "CompactIri::is_xsd_string",
		default = "CompactIri::xsd_string"
	)]
	pub type_: CompactIri,
}

impl LiteralTypeIri {
	pub fn resolve(&self, scope: &Scope) -> Result<IriBuf, BuildError> {
		self.type_.resolve(scope)
	}
}

#[derive(
	Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct LiteralTypeLanguage {
	pub language: LangTagBuf,
}
