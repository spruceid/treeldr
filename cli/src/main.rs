use clap::Parser;
use codespan_reporting::term::{
	self,
	termcolor::{ColorChoice, StandardStream},
};
use std::{convert::Infallible, path::PathBuf};
use treeldr_syntax as syntax;

mod source;

type BuildContext<'v> = treeldr_build::Context<'v, source::FileId, syntax::build::Descriptions>;

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
		match files.load(&filename, None) {
			Ok(file_id) => {
				documents.push(Document::TreeLdr(import_treeldr(&files, file_id)));
			}
			Err(e) => {
				log::error!("unable to read file `{}`: {}", filename.display(), e);
				std::process::exit(1);
			}
		}
	}

	match args.command {
		Some(Command::Rdf) => {
			todo!()
		}
		command => {
			use treeldr::reporting::Diagnose;
			use treeldr::vocab::BorrowWithVocabulary;
			let mut vocabulary = treeldr::Vocabulary::new();
			let mut build_context = BuildContext::new(&mut vocabulary);
			build_context.define_xml_types().unwrap();

			for doc in &mut documents {
				if let Err(e) = doc.declare(&mut build_context) {
					let diagnostic = e.with_vocabulary(build_context.vocabulary()).diagnostic();
					let writer = StandardStream::stderr(ColorChoice::Always);
					let config = codespan_reporting::term::Config::default();
					term::emit(&mut writer.lock(), &config, &files, &diagnostic)
						.expect("diagnostic failed");
					std::process::exit(1);
				}
			}

			for doc in documents {
				if let Err(e) = doc.build(&mut build_context) {
					let diagnostic = e.with_vocabulary(build_context.vocabulary()).diagnostic();
					let writer = StandardStream::stderr(ColorChoice::Always);
					let config = codespan_reporting::term::Config::default();
					term::emit(&mut writer.lock(), &config, &files, &diagnostic)
						.expect("diagnostic failed");
					std::process::exit(1);
				}
			}

			match build_context.build() {
				#[allow(unused_variables)]
				Ok(model) => match command {
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
			}
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
	) -> Result<(), syntax::build::Error<source::FileId>> {
		use treeldr_build::Document;
		self.doc.declare(&mut self.local_context, context)
	}

	fn build(
		mut self,
		context: &mut BuildContext,
	) -> Result<(), syntax::build::Error<source::FileId>> {
		use treeldr_build::Document;
		self.doc.relate(&mut self.local_context, context)
	}
}

pub enum Document {
	TreeLdr(TreeLdrDocument),
}

impl Document {
	fn declare(
		&mut self,
		context: &mut BuildContext,
	) -> Result<(), syntax::build::Error<source::FileId>> {
		match self {
			Self::TreeLdr(d) => d.declare(context),
		}
	}

	fn build(self, context: &mut BuildContext) -> Result<(), syntax::build::Error<source::FileId>> {
		match self {
			Self::TreeLdr(d) => d.build(context),
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
