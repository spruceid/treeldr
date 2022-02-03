use clap::Parser;
use codespan_reporting::term::{
	self,
	termcolor::{ColorChoice, StandardStream},
};
use iref::IriBuf;
use std::{convert::Infallible, fs, io, path::PathBuf};
use treeldr::{error::Diagnose, source, syntax, syntax::Parse, Compile};

#[derive(Parser)]
#[clap(name="treeldr", author, version, about, long_about = None)]
struct Args {
	/// Base IRI.
	base_iri: IriBuf,

	/// Input TreeLDR file.
	#[clap(name = "FILE")]
	filename: PathBuf,

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
}

fn main() -> io::Result<()> {
	// Parse options.
	let args = Args::parse();

	// Init logger.
	stderrlog::new().verbosity(args.verbosity).init().unwrap();

	let content = fs::read_to_string(&args.filename)?;

	let mut files = source::Files::new();
	let (source_id, file) = files.add(source::Path::Local(args.filename), content);

	let mut lexer =
		syntax::Lexer::<Infallible, _>::new(source_id, file.buffer().chars().map(Result::Ok));

	// for token in lexer {
	// 	eprintln!("token: {:?}", token)
	// }

	log::debug!("ready for parsing.");
	match syntax::Document::parse(&mut lexer) {
		Ok(doc) => {
			log::debug!("parsing succeeded.");
			let mut model = treeldr::Model::new(args.base_iri);
			model.define_xml_types().unwrap();

			let mut env = treeldr::compile::Environment::new(&mut model);
			match doc.compile(&mut env) {
				Ok(()) => {
					log::debug!("compilation succeeded.");
					match args.command {
						#[cfg(feature = "json-schema")]
						Some(Command::JsonSchema(command)) => command.execute(&model),
						_ => (),
					}
				}
				Err(e) => {
					let diagnostic = e.with_model(&model).diagnostic();
					let writer = StandardStream::stderr(ColorChoice::Always);
					let config = codespan_reporting::term::Config::default();
					term::emit(&mut writer.lock(), &config, &files, &diagnostic)
						.expect("diagnostic failed");
					std::process::exit(1);
				}
			}
		}
		Err(e) => {
			let diagnostic = e.diagnostic();
			let writer = StandardStream::stderr(ColorChoice::Always);
			let config = codespan_reporting::term::Config::default();
			term::emit(&mut writer.lock(), &config, &files, &diagnostic)
				.expect("diagnostic failed");
			std::process::exit(1);
		}
	}

	Ok(())
}
