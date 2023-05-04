use crate::{source, LangError};

#[cfg(feature = "json-schema")]
impl From<treeldr_json_schema::import::Error<source::Metadata>> for LangError {
	fn from(e: treeldr_json_schema::import::Error<source::Metadata>) -> Self {
		Self::JsonSchema(e)
	}
}

#[cfg(feature = "json-schema")]
pub fn import(
	json: json_syntax::MetaValue<source::Metadata>
) -> treeldr_json_schema::Schema {
	treeldr_json_schema::Schema::try_from(json).expect("invalid JSON Schema")
}