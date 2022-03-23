//! JSON Schema import functions.
//! 
//! Semantics follows <https://www.w3.org/2019/wot/json-schema>.
use serde_json_schema::{
	Schema,
	// id::SchemaId
};
use locspan::{
	Location,
	Span
};
// use iref::IriRefBuf;
use treeldr::{
	vocab,
	Vocabulary
};

/// Import error.
pub enum Error {
	InvalidJSONSchema(serde_json::error::Error)
}

impl From<serde_json::error::Error> for Error {
	fn from(e: serde_json::error::Error) -> Self {
		Self::InvalidJSONSchema(e)
	}
}

/// Create a dummy location.
fn loc<F: Clone>(file: &F) -> Location<F> {
	Location::new(file.clone(), Span::default())
}

pub fn import<F: Clone>(content: &str, file: F, vocabulary: &mut Vocabulary, quads: &mut Vec<vocab::LocQuad<F>>) -> Result<(), Error> {
	let schema: Schema = content.try_into()?;

	import_schema(&schema, &file, vocabulary, quads);

	Ok(())
}

pub fn import_schema<F: Clone>(
	schema: &Schema,
	file: &F,
	vocabulary: &mut Vocabulary,
	quads: &mut Vec<treeldr::vocab::LocQuad<F>>
) {
	// ...
}