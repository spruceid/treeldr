use codespan_reporting::term::{
	self,
	termcolor::{ColorChoice, StandardStream},
};
use locspan::{Location, MaybeLocated, Meta};
use rdf_types::{Generator, Vocabulary, VocabularyMut};
use std::hash::Hash;
use std::path::{Path, PathBuf};
use thiserror::Error;
use treeldr::{
	reporting::Diagnose,
	vocab::{GraphLabel, Object},
	BlankIdIndex, Id, IriIndex,
};
use treeldr_syntax as syntax;

mod source;
pub use source::*;

pub use treeldr::reporting;
pub type BuildContext = treeldr_build::Context<source::Metadata>;

/// Build all the given documents.
pub fn build_all<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
	vocabulary: &mut V,
	generator: &mut impl Generator<V>,
	build_context: &mut BuildContext,
	documents: Vec<Document>,
) -> Result<treeldr::Model<source::Metadata>, BuildAllError> {
	build_context.apply_built_in_definitions(vocabulary, generator);

	let mut declared_documents = Vec::with_capacity(documents.len());
	for doc in documents {
		declared_documents.push(
			doc.declare(build_context, vocabulary, generator)
				.map_err(BuildAllError::Declaration)?,
		)
	}

	for doc in declared_documents {
		doc.build(build_context, vocabulary, generator)
			.map_err(BuildAllError::Link)?
	}

	build_context
		.build(vocabulary, generator)
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
	doc: syntax::Document<source::Metadata>,
	local_context: syntax::build::LocalContext<source::Metadata>,
}

impl TreeLdrDocument {
	fn declare<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&mut self,
		context: &mut BuildContext,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<(), syntax::build::Error<source::Metadata>> {
		use treeldr_build::Document;
		self.doc
			.declare(&mut self.local_context, context, vocabulary, generator)
	}

	fn build<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		mut self,
		context: &mut BuildContext,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<(), syntax::build::Error<source::Metadata>> {
		use treeldr_build::Document;
		self.doc
			.define(&mut self.local_context, context, vocabulary, generator)
	}
}

pub enum BuildAllError {
	Declaration(LangError),
	Link(LangError),
	Build(treeldr_build::Error<source::Metadata>),
}

impl treeldr::reporting::DiagnoseWithVocabulary<source::Metadata> for BuildAllError {
	fn message(
		&self,
		vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
	) -> String {
		match self {
			Self::Declaration(e) => e.message(vocabulary),
			Self::Link(e) => e.message(vocabulary),
			Self::Build(e) => e.message(vocabulary),
		}
	}

	fn labels(
		&self,
		vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
	) -> Vec<codespan_reporting::diagnostic::Label<source::FileId>> {
		match self {
			Self::Declaration(e) => e.labels(vocabulary),
			Self::Link(e) => e.labels(vocabulary),
			Self::Build(e) => e.labels(vocabulary),
		}
	}

	fn notes(
		&self,
		vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
	) -> Vec<String> {
		match self {
			Self::Declaration(e) => e.notes(vocabulary),
			Self::Link(e) => e.notes(vocabulary),
			Self::Build(e) => e.notes(vocabulary),
		}
	}
}

pub enum LangError {
	TreeLdr(syntax::build::Error<source::Metadata>),
	NQuads(treeldr_build::Error<source::Metadata>),
	#[cfg(feature = "turtle")]
	Turtle(turtle_syntax::build::MetaError<source::Metadata>),
	#[cfg(feature = "json-schema")]
	JsonSchema(treeldr_json_schema::import::Error<source::Metadata>),
}

impl From<syntax::build::Error<source::Metadata>> for LangError {
	fn from(e: syntax::build::Error<source::Metadata>) -> Self {
		Self::TreeLdr(e)
	}
}

#[cfg(feature = "turtle")]
impl From<turtle_syntax::build::MetaError<source::Metadata>> for LangError {
	fn from(e: turtle_syntax::build::MetaError<source::Metadata>) -> Self {
		Self::Turtle(e)
	}
}

#[cfg(feature = "json-schema")]
impl From<treeldr_json_schema::import::Error<source::Metadata>> for LangError {
	fn from(e: treeldr_json_schema::import::Error<source::Metadata>) -> Self {
		Self::JsonSchema(e)
	}
}

impl treeldr::reporting::DiagnoseWithVocabulary<source::Metadata> for LangError {
	fn message(
		&self,
		vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
	) -> String {
		match self {
			Self::TreeLdr(e) => e.message(vocabulary),
			Self::NQuads(e) => e.message(vocabulary),
			#[cfg(feature = "turtle")]
			Self::Turtle(e) => e.to_string(),
			#[cfg(feature = "json-schema")]
			Self::JsonSchema(e) => e.message(vocabulary),
		}
	}

	fn labels(
		&self,
		vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
	) -> Vec<codespan_reporting::diagnostic::Label<source::FileId>> {
		match self {
			Self::TreeLdr(e) => e.labels(vocabulary),
			Self::NQuads(e) => e.labels(vocabulary),
			#[cfg(feature = "turtle")]
			Self::Turtle(_) => Vec::new(),
			#[cfg(feature = "json-schema")]
			Self::JsonSchema(e) => e.labels(vocabulary),
		}
	}

	fn notes(
		&self,
		vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
	) -> Vec<String> {
		match self {
			Self::TreeLdr(e) => e.notes(vocabulary),
			Self::NQuads(e) => e.notes(vocabulary),
			#[cfg(feature = "turtle")]
			Self::Turtle(_) => Vec::new(),
			#[cfg(feature = "json-schema")]
			Self::JsonSchema(e) => e.notes(vocabulary),
		}
	}
}

pub enum Document {
	TreeLdr(Box<TreeLdrDocument>),

	NQuads(nquads_syntax::Document<source::Metadata>),

	#[cfg(feature = "turtle")]
	Turtle(turtle_syntax::Document<source::Metadata>),

	#[cfg(feature = "json-schema")]
	JsonSchema(Box<treeldr_json_schema::Schema>),
}

pub enum DeclaredDocument {
	TreeLdr(Box<TreeLdrDocument>),

	NQuads(Dataset),

	#[cfg(feature = "turtle")]
	Turtle(Dataset),

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
					Some(source::MimeType::NQuads) => {
						Document::NQuads(import_nquads(files, file_id))
					}
					#[cfg(feature = "json-schema")]
					Some(source::MimeType::JsonSchema) => {
						Document::JsonSchema(Box::new(import_json_schema(files, file_id)))
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

	fn declare<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		context: &mut BuildContext,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<DeclaredDocument, LangError> {
		match self {
			Self::TreeLdr(mut d) => {
				d.declare(context, vocabulary, generator)?;
				Ok(DeclaredDocument::TreeLdr(d))
			}
			Self::NQuads(d) => {
				let dataset: Dataset = d
					.into_iter()
					.map(|Meta(quad, meta)| {
						Meta(
							quad.insert_into(vocabulary)
								.map_predicate(|Meta(p, m)| Meta(Id::Iri(p), m)),
							meta,
						)
					})
					.collect();

				use treeldr_build::Document;
				dataset
					.declare(&mut (), context, vocabulary, generator)
					.map_err(LangError::NQuads)?;
				Ok(DeclaredDocument::NQuads(dataset))
			}
			#[cfg(feature = "turtle")]
			Self::Turtle(d) => {
				let dataset: Dataset = d
					.build_triples_with(None, vocabulary, &mut *generator)?
					.into_iter()
					.map(|Meta(triple, meta)| {
						Meta(
							triple
								.map_predicate(|Meta(p, m)| Meta(Id::Iri(p), m))
								.into_quad(None),
							meta,
						)
					})
					.collect();

				use treeldr_build::Document;
				dataset
					.declare(&mut (), context, vocabulary, generator)
					.map_err(LangError::NQuads)?;
				Ok(DeclaredDocument::NQuads(dataset))
			}
			#[cfg(feature = "json-schema")]
			Self::JsonSchema(s) => {
				treeldr_json_schema::import_schema(&s, None, context, vocabulary, generator)?;
				Ok(DeclaredDocument::JsonSchema(s))
			}
		}
	}
}

impl DeclaredDocument {
	fn build<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		context: &mut BuildContext,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<(), LangError> {
		match self {
			Self::TreeLdr(d) => {
				d.build(context, vocabulary, generator)?;
				Ok(())
			}
			Self::NQuads(d) => {
				use treeldr_build::Document;
				d.define(&mut (), context, vocabulary, generator)
					.map_err(LangError::NQuads)?;
				Ok(())
			}
			#[cfg(feature = "turtle")]
			Self::Turtle(d) => {
				use treeldr_build::Document;
				d.define(&mut (), context, vocabulary, generator)
					.map_err(LangError::NQuads)?;
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
	let file = files.get(source_id).unwrap();

	log::debug!("ready for parsing.");
	match syntax::Document::parse_str(file.buffer().as_str(), |span| {
		source::Metadata::Extern(Location::new(source_id, span))
	}) {
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

/// RDF dataset.
pub type Dataset =
	grdf::meta::BTreeDataset<Id, Id, Object<source::Metadata>, GraphLabel, source::Metadata>;

/// Import a N-Quads file.
pub fn import_nquads<'f, P>(
	files: &'f source::Files<P>,
	source_id: source::FileId,
) -> nquads_syntax::Document<source::Metadata>
where
	P: DisplayPath<'f>,
{
	use nquads_syntax::Parse;
	let file = files.get(source_id).unwrap();
	match nquads_syntax::Document::parse_str(file.buffer().as_str(), |span| {
		source::Metadata::Extern(Location::new(source_id, span))
	}) {
		Ok(Meta(doc, _)) => {
			log::debug!("parsing succeeded.");
			doc
		}
		Err(Meta(e, meta)) => {
			let diagnostic = codespan_reporting::diagnostic::Diagnostic::error()
				.with_message("parse error")
				.with_labels(vec![meta
					.optional_location()
					.unwrap()
					.as_primary_label()
					.with_message(e.to_string())]);
			let writer = StandardStream::stderr(ColorChoice::Always);
			let config = codespan_reporting::term::Config::default();
			term::emit(&mut writer.lock(), &config, files, &diagnostic).expect("diagnostic failed");
			std::process::exit(1);
		}
	}
}

#[cfg(feature = "json-schema")]
pub fn import_json_schema<P>(
	files: &source::Files<P>,
	source_id: source::FileId,
) -> treeldr_json_schema::Schema {
	let file = files.get(source_id).unwrap();
	let json: serde_json::Value = serde_json::from_str(file.buffer()).expect("invalid JSON");
	treeldr_json_schema::Schema::try_from(json).expect("invalid JSON Schema")
}
