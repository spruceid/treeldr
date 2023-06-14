use std::path::PathBuf;

use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use locspan::{MaybeLocated, Meta};
use thiserror::Error;
use treeldr::{reporting::Diagnose, vocab::TldrVocabulary};

use crate::source;

#[derive(Error, Debug)]
pub enum LoadError {
	#[error("unsupported MIME type `{0}`")]
	UnsupportedMimeType(source::MimeType),

	#[error("unrecognized format for file `{0}`")]
	UnrecognizedFormat(PathBuf),

	#[error("unable to read file `{0}`: {1}")]
	UnableToRead(PathBuf, std::io::Error),

	#[error("parse error")]
	Parsing(#[from] Meta<ParseError, source::Metadata>),
}

#[derive(Debug)]
pub enum BuildAllError {
	Declaration(LangError),
	Link(LangError),
	Build(treeldr_build::Error<source::Metadata>),
}

impl treeldr::reporting::DiagnoseWithVocabulary<source::Metadata> for BuildAllError {
	fn message(&self, vocabulary: &TldrVocabulary) -> String {
		match self {
			Self::Declaration(e) => e.message(vocabulary),
			Self::Link(e) => e.message(vocabulary),
			Self::Build(e) => e.message(vocabulary),
		}
	}

	fn labels(
		&self,
		vocabulary: &TldrVocabulary,
	) -> Vec<codespan_reporting::diagnostic::Label<source::FileId>> {
		match self {
			Self::Declaration(e) => e.labels(vocabulary),
			Self::Link(e) => e.labels(vocabulary),
			Self::Build(e) => e.labels(vocabulary),
		}
	}

	fn notes(&self, vocabulary: &TldrVocabulary) -> Vec<String> {
		match self {
			Self::Declaration(e) => e.notes(vocabulary),
			Self::Link(e) => e.notes(vocabulary),
			Self::Build(e) => e.notes(vocabulary),
		}
	}
}

#[derive(Debug)]
pub enum LangError {
	TreeLdr(treeldr_syntax::build::Error<source::Metadata>),
	NQuads(treeldr_build::Error<source::Metadata>),
	#[cfg(feature = "turtle")]
	Turtle(turtle_syntax::build::MetaError<source::Metadata>),
	#[cfg(feature = "json-schema")]
	JsonSchema(treeldr_json_schema::import::Error<source::Metadata>),
}

impl treeldr::reporting::DiagnoseWithVocabulary<source::Metadata> for LangError {
	fn message(&self, vocabulary: &TldrVocabulary) -> String {
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
		vocabulary: &TldrVocabulary,
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

	fn notes(&self, vocabulary: &TldrVocabulary) -> Vec<String> {
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

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
	#[error("TreeLDR syntax error")]
	TreeLdr(Box<treeldr_syntax::parsing::Error<treeldr_syntax::lexing::Error>>),

	#[error("Turtle syntax error")]
	Turtle(Box<turtle_syntax::parsing::Error<turtle_syntax::lexing::Error>>),

	#[error("N-Quads syntax error")]
	NQuads(Box<nquads_syntax::parsing::Error<nquads_syntax::lexing::Error>>),
}

impl ParseError {
	pub fn diagnostic(
		self,
		meta: source::Metadata,
	) -> codespan_reporting::diagnostic::Diagnostic<source::FileId> {
		match self {
			Self::TreeLdr(e) => Meta(e, meta).diagnostic(),
			Self::NQuads(e) => codespan_reporting::diagnostic::Diagnostic::error()
				.with_message("parse error")
				.with_labels(vec![meta
					.optional_location()
					.unwrap()
					.as_primary_label()
					.with_message(e.to_string())]),
			Self::Turtle(e) => codespan_reporting::diagnostic::Diagnostic::error()
				.with_message("parse error")
				.with_labels(vec![meta
					.optional_location()
					.unwrap()
					.as_primary_label()
					.with_message(e.to_string())]),
		}
	}

	pub fn display_and_exit<'a, P: source::DisplayPath<'a>>(
		self,
		files: &'a source::Files<P>,
		meta: source::Metadata,
	) {
		let diagnostic = self.diagnostic(meta);
		let writer = StandardStream::stderr(ColorChoice::Always);
		let config = codespan_reporting::term::Config::default();
		codespan_reporting::term::emit(&mut writer.lock(), &config, files, &diagnostic)
			.expect("diagnostic failed");
		std::process::exit(1);
	}
}
