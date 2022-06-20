use clap::Parser;
use codespan_reporting::term::{
	self,
	termcolor::{ColorChoice, StandardStream},
};
use std::{convert::Infallible, path::PathBuf};
use treeldr::Vocabulary;
use treeldr_syntax as syntax;

mod source;

type BuildContext = treeldr_build::Context<source::FileId, syntax::build::Descriptions>;

#[derive(Parser)]
#[clap(name="treeldr", author, version, about, long_about = None)]
struct Args {
	/// Input files.
	#[clap(short = 'i', multiple_occurrences = true)]
	filenames: Vec<PathBuf>,

	/// Sets the level of verbosity.
	#[clap(short, long = "verbose", parse(from_occurrences))]
	verbosity: usize,

	#[clap(subcommand)]
	command: Option<Command>,
}

#[derive(clap::Subcommand)]
pub enum Command {
	/// Convert the input to an RDF dataset.
	Rdf,

	#[cfg(feature = "json-schema")]
	JsonSchema(treeldr_json_schema::Command),

	#[cfg(feature = "json-ld-context")]
	JsonLdContext(treeldr_json_ld_context::Command),
}

fn main() {
	// Parse options.
	let args = Args::parse();

	// Init logger.
	stderrlog::new().verbosity(args.verbosity).init().unwrap();

	let mut files = source::Files::new();
	let mut documents = Vec::new();
	for filename in args.filenames {
		match files.load(&filename, None, None) {
			Ok(file_id) => {
				let document = match files.get(file_id).unwrap().mime_type() {
					Some(source::MimeType::TreeLdr) => {
						Document::TreeLdr(Box::new(import_treeldr(&files, file_id)))
					}
					#[cfg(feature = "json-schema")]
					Some(source::MimeType::JsonSchema) => {
						Document::JsonSchema(Box::new(import_json_schema(&files, file_id)))
					}
					#[allow(unreachable_patterns)]
					Some(mime_type) => {
						log::error!(
							"unsupported mime type `{}` for file `{}`",
							mime_type,
							filename.display()
						);
						std::process::exit(1);
					}
					None => {
						log::error!("unknown format for file `{}`", filename.display());
						std::process::exit(1);
					}
				};

				documents.push(document)
			}
			Err(e) => {
				log::error!("unable to read file `{}`: {}", filename.display(), e);
				std::process::exit(1);
			}
		}
	}

	use treeldr::reporting::Diagnose;
	use treeldr::vocab::BorrowWithVocabulary;
	let mut vocabulary = treeldr::Vocabulary::new();
	let mut build_context = BuildContext::new();
	build_context.apply_built_in_definitions(&mut vocabulary).unwrap();

	for doc in &mut documents {
		if let Err(e) = doc.declare(&mut build_context, &mut vocabulary) {
			let diagnostic = e.with_vocabulary(&vocabulary).diagnostic();
			let writer = StandardStream::stderr(ColorChoice::Always);
			let config = codespan_reporting::term::Config::default();
			term::emit(&mut writer.lock(), &config, &files, &diagnostic)
				.expect("diagnostic failed");
			std::process::exit(1);
		}
	}

	for doc in documents {
		if let Err(e) = doc.build(&mut build_context, &mut vocabulary) {
			let diagnostic = e.with_vocabulary(&vocabulary).diagnostic();
			let writer = StandardStream::stderr(ColorChoice::Always);
			let config = codespan_reporting::term::Config::default();
			term::emit(&mut writer.lock(), &config, &files, &diagnostic)
				.expect("diagnostic failed");
			std::process::exit(1);
		}
	}

	match build_context.simplify(&mut vocabulary) {
		Ok(build_context) => match build_context.build(&mut vocabulary) {
			#[allow(unused_variables)]
			Ok(model) => match args.command {
				Some(Command::Rdf) => {
					use treeldr::vocab::RdfDisplay;
					let mut quads = Vec::new();
					model.to_rdf(&mut vocabulary, &mut quads);
					for quad in quads {
						println!("{} .", quad.rdf_display(&vocabulary))
					}
				}
				#[cfg(feature = "json-schema")]
				Some(Command::JsonSchema(command)) => command.execute(&vocabulary, &model),
				#[cfg(feature = "json-ld-context")]
				Some(Command::JsonLdContext(command)) => command.execute(&vocabulary, &model),
				_ => (),
			},
			Err(e) => {
				let diagnostic = e.with_vocabulary(&vocabulary).diagnostic();
				let writer = StandardStream::stderr(ColorChoice::Always);
				let config = codespan_reporting::term::Config::default();
				term::emit(&mut writer.lock(), &config, &files, &diagnostic)
					.expect("diagnostic failed");
				std::process::exit(1);
			}
		},
		Err(e) => {
			let diagnostic = e.with_vocabulary(&vocabulary).diagnostic();
			let writer = StandardStream::stderr(ColorChoice::Always);
			let config = codespan_reporting::term::Config::default();
			term::emit(&mut writer.lock(), &config, &files, &diagnostic)
				.expect("diagnostic failed");
			std::process::exit(1);
		}
	}
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

pub enum Error {
	TreeLdr(syntax::build::Error<source::FileId>),
	#[cfg(feature = "json-schema")]
	JsonSchema(treeldr_json_schema::import::Error<source::FileId>),
}

impl From<syntax::build::Error<source::FileId>> for Error {
	fn from(e: syntax::build::Error<source::FileId>) -> Self {
		Self::TreeLdr(e)
	}
}

#[cfg(feature = "json-schema")]
impl From<treeldr_json_schema::import::Error<source::FileId>> for Error {
	fn from(e: treeldr_json_schema::import::Error<source::FileId>) -> Self {
		Self::JsonSchema(e)
	}
}

impl treeldr::reporting::DiagnoseWithVocabulary<source::FileId> for Error {
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
	fn declare(
		&mut self,
		context: &mut BuildContext,
		vocabulary: &mut Vocabulary,
	) -> Result<(), Error> {
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

	fn build(self, context: &mut BuildContext, vocabulary: &mut Vocabulary) -> Result<(), Error> {
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
fn import_treeldr(files: &source::Files, source_id: source::FileId) -> TreeLdrDocument {
	use syntax::Parse;
	use treeldr::reporting::Diagnose;
	let file = files.get(source_id).unwrap();

	let mut lexer =
		syntax::Lexer::<_, Infallible, _>::new(source_id, file.buffer().chars().map(Result::Ok));

	log::debug!("ready for parsing.");
	match syntax::Document::parse(&mut lexer) {
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
fn import_json_schema(
	files: &source::Files,
	source_id: source::FileId,
) -> treeldr_json_schema::Schema {
	let file = files.get(source_id).unwrap();
	let json: serde_json::Value = serde_json::from_str(file.buffer()).expect("invalid JSON");
	treeldr_json_schema::Schema::try_from(json).expect("invalid JSON Schema")
}
