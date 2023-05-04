use json_syntax::Parse;
use locspan::Location;
use rdf_types::{Generator, VocabularyMut};
use treeldr::{IriIndex, BlankIdIndex};

use crate::{LoadError, source, BuildContext, LangError};

#[cfg(feature = "json-schema")]
pub mod schema;

#[cfg(feature = "lexicon")]
pub mod lexicon;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MimeType {
	/// application/schema+json
	JsonSchema,

	/// application/lexicon+json
	Lexicon
}

impl MimeType {
	pub fn name(&self) -> &'static str {
		match self {
			Self::JsonSchema => "application/schema+json",
			Self::Lexicon => "application/lexicon+json"
		}
	}

	pub fn infer(json: &json_syntax::MetaValue<source::Metadata>) -> Option<Self> {
		#[cfg(feature = "json-schema")]
		if treeldr_json_schema::import::is_json_schema(json) {
			return Some(Self::JsonSchema)
		}

		#[cfg(feature = "lexicon")]
		if treeldr_json_schema::import::is_lexicon_document(json) {
			return Some(Self::Lexicon)
		}

		None
	}
}

pub enum Document {
	#[cfg(feature = "json-schema")]
	Schema(treeldr_json_schema::Schema),

	#[cfg(feature = "lexicon")]
	Lexicon
}

impl Document {
	pub fn declare<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &mut BuildContext,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<(), LangError> {
		match self {
			Self::Schema(s) => {
				treeldr_json_schema::import_schema(&s, None, context, vocabulary, generator)?;
				Ok(())
			}
			#[cfg(feature = "lexicon")]
			Self::Lexicon => {
				todo!()
			}
		}
	}
}

pub fn import<P>(
	files: &source::Files<P>,
	source_id: source::FileId,
	mime_type: Option<MimeType>
) -> Result<Document, LoadError> {
	let file = files.get(source_id).unwrap();
	let json = json_syntax::Value::parse_str(file.buffer(), |span| source::Metadata::Extern(Location::new(source_id, span))).expect("invalid JSON");

	match mime_type.or_else(|| MimeType::infer(&json)) {
		Some(MimeType::JsonSchema) => {
			Ok(Document::Schema(schema::import(json)))
		}
		#[cfg(feature = "lexicon")]
		Some(MimeType::Lexicon) => {
			Ok(Document::Schema(schema::import(json)))
		}
		unsupported => Err(LoadError::UnsupportedMimeType(crate::MimeType::Json(unsupported)))
	}
}