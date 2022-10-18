use codespan_reporting::term::{
	self,
	termcolor::{ColorChoice, StandardStream},
};
use std::convert::Infallible;
use std::hash::Hash;
use std::path::{Path, PathBuf};
use thiserror::Error;
use treeldr_syntax as syntax;

mod source;
pub use source::*;

pub use treeldr::{reporting, vocab::BorrowWithVocabulary, Vocabulary};
pub type BuildContext = treeldr_build::Context<source::FileId, syntax::build::Descriptions>;

/// Build all the given documents.
pub fn build_all(
	vocabulary: &mut treeldr::Vocabulary,
	build_context: &mut BuildContext,
	mut documents: Vec<Document>,
) -> Result<treeldr::Model<source::FileId>, BuildAllError> {
	build_context
		.apply_built_in_definitions(vocabulary)
		.unwrap();

	for doc in &mut documents {
		doc.declare(build_context, vocabulary)
			.map_err(BuildAllError::Declaration)?
	}

	for doc in documents {
		doc.build(build_context, vocabulary)
			.map_err(BuildAllError::Link)?
	}

	let build_context = build_context
		.simplify(vocabulary)
		.map_err(BuildAllError::Simplification)?;
	build_context
		.build(vocabulary)
		.map_err(BuildAllError::Build)
}

#[derive(Error, Debug)]
pub enum LoadError {
	#[error("unsupported MIME type `{0}`")]
	UnsupportedMimeType(source::MimeType),

	#[error("unrecognized format for file `{0}`")]
	UnrecognizedFormat(PathBuf),

	#[error("unable to read file `{0}`: {1}")]
	UnableToRead(PathBuf, std::io::Error),
}

pub struct TreeLdrDocument {
	doc: syntax::Document<source::FileId>,
	local_context: syntax::build::LocalContext<source::FileId>,
}

impl TreeLdrDocument {
	fn declare(
		&mut self,
		context: &mut BuildContext,
		vocabulary: &mut Vocabulary,
	) -> Result<(), syntax::build::Error<source::FileId>> {
		use treeldr_build::Document;
		self.doc
			.declare(&mut self.local_context, context, vocabulary)
	}

	fn build(
		mut self,
		context: &mut BuildContext,
		vocabulary: &mut Vocabulary,
	) -> Result<(), syntax::build::Error<source::FileId>> {
		use treeldr_build::Document;
		self.doc
			.relate(&mut self.local_context, context, vocabulary)
	}
}

pub enum BuildAllError {
	Declaration(LangError),
	Link(LangError),
	Simplification(<syntax::build::Descriptions as treeldr_build::Simplify<source::FileId>>::Error),
	Build(treeldr_build::Error<source::FileId>),
}

impl treeldr::reporting::DiagnoseWithVocabulary<source::FileId> for BuildAllError {
	fn message(&self, vocabulary: &Vocabulary) -> String {
		match self {
			Self::Declaration(e) => e.message(vocabulary),
			Self::Link(e) => e.message(vocabulary),
			Self::Simplification(e) => e.message(vocabulary),
			Self::Build(e) => e.message(vocabulary),
		}
	}

	fn labels(
		&self,
		vocabulary: &Vocabulary,
	) -> Vec<codespan_reporting::diagnostic::Label<source::FileId>> {
		match self {
			Self::Declaration(e) => e.labels(vocabulary),
			Self::Link(e) => e.labels(vocabulary),
			Self::Simplification(e) => e.labels(vocabulary),
			Self::Build(e) => e.labels(vocabulary),
		}
	}

	fn notes(&self, vocabulary: &Vocabulary) -> Vec<String> {
		match self {
			Self::Declaration(e) => e.notes(vocabulary),
			Self::Link(e) => e.notes(vocabulary),
			Self::Simplification(e) => e.notes(vocabulary),
			Self::Build(e) => e.notes(vocabulary),
		}
	}
}

pub enum LangError {
	TreeLdr(syntax::build::Error<source::FileId>),
	#[cfg(feature = "json-schema")]
	JsonSchema(treeldr_json_schema::import::Error<source::FileId>),
}

impl From<syntax::build::Error<source::FileId>> for LangError {
	fn from(e: syntax::build::Error<source::FileId>) -> Self {
		Self::TreeLdr(e)
	}
}

#[cfg(feature = "json-schema")]
impl From<treeldr_json_schema::import::Error<source::FileId>> for LangError {
	fn from(e: treeldr_json_schema::import::Error<source::FileId>) -> Self {
		Self::JsonSchema(e)
	}
}

impl treeldr::reporting::DiagnoseWithVocabulary<source::FileId> for LangError {
	fn message(&self, vocabulary: &Vocabulary) -> String {
		match self {
			Self::TreeLdr(e) => e.message(vocabulary),
			#[cfg(feature = "json-schema")]
			Self::JsonSchema(e) => e.message(vocabulary),
		}
	}

	fn labels(
		&self,
		vocabulary: &Vocabulary,
	) -> Vec<codespan_reporting::diagnostic::Label<source::FileId>> {
		match self {
			Self::TreeLdr(e) => e.labels(vocabulary),
			#[cfg(feature = "json-schema")]
			Self::JsonSchema(e) => e.labels(vocabulary),
		}
	}

	fn notes(&self, vocabulary: &Vocabulary) -> Vec<String> {
		match self {
			Self::TreeLdr(e) => e.notes(vocabulary),
			#[cfg(feature = "json-schema")]
			Self::JsonSchema(e) => e.notes(vocabulary),
		}
	}
}

pub enum Document {
	TreeLdr(Box<TreeLdrDocument>),

	#[cfg(feature = "json-schema")]
	JsonSchema(Box<treeldr_json_schema::Schema>),
}

impl Document {
	/// Load the document located at the given `path`.
	pub fn load<'f, P>(
		files: &'f mut source::Files<P>,
		filename: &Path,
	) -> Result<(Self, source::FileId), LoadError>
	where
		P: Clone + Eq + Hash + DisplayPath<'f> + for<'a> From<&'a Path>,
	{
		match files.load(&filename, None, None) {
			Ok(file_id) => {
				let document = match files.get(file_id).unwrap().mime_type() {
					Some(source::MimeType::TreeLdr) => {
						Document::TreeLdr(Box::new(import_treeldr(files, file_id)))
					}
					#[cfg(feature = "json-schema")]
					Some(source::MimeType::JsonSchema) => {
						Document::JsonSchema(Box::new(import_json_schema(&files, file_id)))
					}
					#[allow(unreachable_patterns)]
					Some(mime_type) => return Err(LoadError::UnsupportedMimeType(mime_type)),
					None => return Err(LoadError::UnrecognizedFormat(filename.to_owned())),
				};

				Ok((document, file_id))
			}
			Err(e) => Err(LoadError::UnableToRead(filename.to_owned(), e)),
		}
	}

	fn declare(
		&mut self,
		context: &mut BuildContext,
		vocabulary: &mut Vocabulary,
	) -> Result<(), LangError> {
		match self {
			Self::TreeLdr(d) => {
				d.declare(context, vocabulary)?;
				Ok(())
			}
			#[cfg(feature = "json-schema")]
			Self::JsonSchema(s) => {
				treeldr_json_schema::import_schema(s, None, context, vocabulary)?;
				Ok(())
			}
		}
	}

	fn build(
		self,
		context: &mut BuildContext,
		vocabulary: &mut Vocabulary,
	) -> Result<(), LangError> {
		match self {
			Self::TreeLdr(d) => {
				d.build(context, vocabulary)?;
				Ok(())
			}
			#[cfg(feature = "json-schema")]
			Self::JsonSchema(_) => Ok(()),
		}
	}
}

/// Import a TreeLDR file.
pub fn import_treeldr<'f, P>(
	files: &'f source::Files<P>,
	source_id: source::FileId,
) -> TreeLdrDocument
where
	P: DisplayPath<'f>,
{
	use syntax::Parse;
	use treeldr::reporting::Diagnose;
	let file = files.get(source_id).unwrap();

	let mut lexer =
		syntax::Lexer::<_, Infallible, _>::new(source_id, file.buffer().chars().map(Result::Ok));

	log::debug!("ready for parsing.");
	match syntax::Document::parse_in(&mut lexer) {
		Ok(doc) => {
			log::debug!("parsing succeeded.");
			TreeLdrDocument {
				doc: doc.into_value(),
				local_context: syntax::build::LocalContext::new(
					file.base_iri().map(|iri| iri.into()),
				),
			}
		}
		Err(e) => {
			let diagnostic = e.diagnostic();
			let writer = StandardStream::stderr(ColorChoice::Always);
			let config = codespan_reporting::term::Config::default();
			term::emit(&mut writer.lock(), &config, files, &diagnostic).expect("diagnostic failed");
			std::process::exit(1);
		}
	}
}

#[cfg(feature = "json-schema")]
pub fn import_json_schema(
	files: &source::Files,
	source_id: source::FileId,
) -> treeldr_json_schema::Schema {
	let file = files.get(source_id).unwrap();
	let json: serde_json::Value = serde_json::from_str(file.buffer()).expect("invalid JSON");
	treeldr_json_schema::Schema::try_from(json).expect("invalid JSON Schema")
}
