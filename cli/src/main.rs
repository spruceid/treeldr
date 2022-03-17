use clap::Parser;
use codespan_reporting::term::{
	self,
	termcolor::{ColorChoice, StandardStream},
};
use std::{convert::Infallible, path::PathBuf};
use treeldr_syntax as syntax;

mod source;

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
	/// Dump the parsed RDF dataset.
	Dump,

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
	let mut vocab = treeldr::Vocabulary::new();
	let mut quads = Vec::new();
	for filename in args.filenames {
		match files.load(&filename, None) {
			Ok(file_id) => {
				import_treeldr(&mut vocab, &mut quads, &files, file_id);
			}
			Err(e) => {
				log::error!("unable to read file `{}`: {}", filename.display(), e);
				std::process::exit(1);
			}
		}
	}

	match args.command {
		Some(Command::Dump) => {
			for quad in &quads {
				use treeldr::vocab::RdfDisplay;
				println!("{} .", quad.rdf_display(&vocab))
			}
		}
		command => {
			let mut build_context: treeldr::build::Context<source::FileId> = treeldr::build::Context::with_vocabulary(vocab);
			build_context.define_xml_types().unwrap();

			match build_context.build_dataset(quads.into_iter().collect()) {
				Ok(model) => {
					match command {
						#[cfg(feature = "json-schema")]
						Some(Command::JsonSchema(command)) => command.execute(&model),
						#[cfg(feature = "json-ld-context")]
						Some(Command::JsonLdContext(command)) => command.execute(&model),
						_ => (),
					}
				}
				Err((e, vocab)) => {
					use treeldr::reporting::Diagnose;
					let diagnostic = e.with_vocabulary(&vocab).diagnostic();
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

/// Import a TreeLDR file.
fn import_treeldr(vocab: &mut treeldr::Vocabulary, quads: &mut Vec<syntax::vocab::LocQuad<source::FileId>>, files: &source::Files, source_id: source::FileId) {
	use syntax::{
		Parse,
		Build,
		reporting::Diagnose
	};
	let file = files.get(source_id).unwrap();

	let mut lexer =
		syntax::Lexer::<_, Infallible, _>::new(source_id, file.buffer().chars().map(Result::Ok));

	log::debug!("ready for parsing.");
	match syntax::Document::parse(&mut lexer) {
		Ok(doc) => {
			log::debug!("parsing succeeded.");
			let mut env = syntax::build::Context::new(vocab, file.base_iri().map(|iri| iri.into()));
			match doc.build(&mut env, quads) {
				Ok(()) => {
					log::debug!("build succeeded.");
				}
				Err(e) => {
					let diagnostic = e.diagnostic();
					let writer = StandardStream::stderr(ColorChoice::Always);
					let config = codespan_reporting::term::Config::default();
					term::emit(&mut writer.lock(), &config, files, &diagnostic)
						.expect("diagnostic failed");
					std::process::exit(1);
				}
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
