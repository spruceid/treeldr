use json_syntax::Parse;
use locspan::{Location, Meta};
use rdf_types::{Generator, Id};
use treeldr::vocab::TldrVocabulary;

use crate::{source, BuildContext, Dataset, LangError, LoadError};

#[cfg(feature = "json-schema")]
pub mod schema;

#[cfg(feature = "lexicon")]
pub mod lexicon;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MimeType {
	/// application/schema+json
	JsonSchema,

	/// application/lexicon+json
	Lexicon,
}

impl MimeType {
	pub fn name(&self) -> &'static str {
		match self {
			Self::JsonSchema => "application/schema+json",
			Self::Lexicon => "application/lexicon+json",
		}
	}

	pub fn infer(json: &json_syntax::MetaValue<source::Metadata>) -> Option<Self> {
		#[cfg(feature = "json-schema")]
		if treeldr_json_schema::import::is_json_schema(json) {
			return Some(Self::JsonSchema);
		}

		#[cfg(feature = "lexicon")]
		if treeldr_lexicon::import::is_lexicon_document(json) {
			return Some(Self::Lexicon);
		}

		None
	}
}

pub enum Document {
	#[cfg(feature = "json-schema")]
	Schema(treeldr_json_schema::Schema),

	#[cfg(feature = "lexicon")]
	Lexicon(treeldr_lexicon::LexiconDoc),
}

impl Document {
	pub fn declare(
		self,
		context: &mut BuildContext,
		vocabulary: &mut TldrVocabulary,
		generator: &mut impl Generator<TldrVocabulary>,
	) -> Result<crate::document::DeclaredDocument, LangError> {
		match self {
			#[cfg(feature = "json-schema")]
			Self::Schema(s) => {
				treeldr_json_schema::import_schema(&s, None, context, vocabulary, generator)?;
				Ok(crate::document::DeclaredDocument::Json(Box::new(
					DeclaredDocument::Schema(s),
				)))
			}
			#[cfg(feature = "lexicon")]
			Self::Lexicon(d) => {
				let dataset: Dataset = d
					.into_triples(vocabulary, &mut *generator)
					.map(|triple| {
						Meta(
							triple
								.map_subject(|s| Meta(s, source::Metadata::default()))
								.map_predicate(|p| Meta(Id::Iri(p), source::Metadata::default()))
								.map_object(|o| Meta(o, source::Metadata::default()))
								.into_quad(None),
							source::Metadata::default(),
						)
					})
					.collect();

				use treeldr_build::Document;
				dataset
					.declare(&mut (), context, vocabulary, generator)
					.map_err(LangError::NQuads)?;
				Ok(crate::document::DeclaredDocument::NQuads(dataset))
			}
		}
	}
}

pub enum DeclaredDocument {
	#[cfg(feature = "json-schema")]
	Schema(treeldr_json_schema::Schema),
}

pub fn import<P>(
	files: &source::Files<P>,
	source_id: source::FileId,
	mime_type: Option<MimeType>,
) -> Result<Document, LoadError> {
	let file = files.get(source_id).unwrap();
	let json = json_syntax::Value::parse_str(file.buffer(), |span| {
		source::Metadata::Extern(Location::new(source_id, span))
	})
	.expect("invalid JSON");

	match mime_type.or_else(|| MimeType::infer(&json)) {
		#[cfg(feature = "json-schema")]
		Some(MimeType::JsonSchema) => Ok(Document::Schema(schema::import(json))),
		#[cfg(feature = "lexicon")]
		Some(MimeType::Lexicon) => Ok(Document::Lexicon(lexicon::import(json))),
		unsupported => Err(LoadError::UnsupportedMimeType(crate::MimeType::Json(
			unsupported,
		))),
	}
}
