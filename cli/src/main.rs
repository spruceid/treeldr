use clap::Parser;
use codespan_reporting::term::{
	self,
	termcolor::{ColorChoice, StandardStream},
};
use iref::IriBuf;
use std::{convert::Infallible, fs, path::PathBuf};
use treeldr::{error::Diagnose, source, syntax, syntax::Parse, Build};

#[derive(Parser)]
#[clap(name="treeldr", author, version, about, long_about = None)]
struct Args {
	/// Base IRI.
	base_iri: IriBuf,

	/// Input files.
	#[clap(short = 'i', multiple_occurrences=true)]
	filenames: Vec<PathBuf>,

	/// Sets the level of verbosity.
	#[clap(short, long = "verbose", parse(from_occurrences))]
	verbosity: usize,

	#[clap(subcommand)]
	command: Option<Command>,
}

#[derive(clap::Subcommand)]
pub enum Command {
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

	let mut model = treeldr::Model::new(args.base_iri);
	model.define_xml_types().unwrap();

	let mut files = source::Files::new();
	for filename in args.filenames {
		match fs::read_to_string(&filename) {
			Ok(content) => {
				let (file_id, _) = files.add(source::Path::Local(filename), content);
				import_treeldr(&mut model, &files, file_id);
			}
			Err(e) => {
				log::error!("unable to read file `{}`: {}", filename.display(), e);
				std::process::exit(1);
			}
		}
	}

	match args.command {
		#[cfg(feature = "json-schema")]
		Some(Command::JsonSchema(command)) => command.execute(&model),
		#[cfg(feature = "json-ld-context")]
		Some(Command::JsonLdContext(command)) => command.execute(&model),
		_ => (),
	}
}

/// Import a TreeLDR file.
fn import_treeldr(
	model: &mut treeldr::Model,
	files: &source::Files,
	source_id: source::Id,
) {
	let file = files.get(source_id).unwrap();

	let mut lexer =
		syntax::Lexer::<Infallible, _>::new(source_id, file.buffer().chars().map(Result::Ok));

	log::debug!("ready for parsing.");
	match syntax::Document::parse(&mut lexer) {
		Ok(doc) => {
			log::debug!("parsing succeeded.");
			let mut env = treeldr::build::Environment::new(model);
			match doc.build(&mut env) {
				Ok(()) => {
					log::debug!("build succeeded.");
				}
				Err(e) => {
					let diagnostic = e.with_model(model).diagnostic();
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
			term::emit(&mut writer.lock(), &config, files, &diagnostic)
				.expect("diagnostic failed");
			std::process::exit(1);
		}
	}
}