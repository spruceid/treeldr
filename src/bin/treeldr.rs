use clap::Parser;
use codespan_reporting::{
	term::{
		self,
		termcolor::{ColorChoice, StandardStream},
	},
};
use iref::IriBuf;
use std::{convert::Infallible, fs, io, path::PathBuf};
use treeldr::{source, syntax, syntax::Parse, Compile, error::Diagnose};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
	/// Base IRI.
	base_iri: IriBuf,

	/// Input TreeLDR file.
	#[clap(name="FILE")]
	filename: PathBuf,

	/// Sets the level of verbosity.
	#[clap(short, long="verbose", parse(from_occurrences))]
	verbosity: usize,
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
		syntax::Lexer::<Infallible, _>::new(source_id, file.buffer().chars().map(Result::Ok))
			.peekable();

	// for token in lexer {
	// 	eprintln!("token: {:?}", token)
	// }

	log::debug!("ready for parsing.");
	match syntax::Document::parse(source_id, &mut lexer, 0) {
		Ok(doc) => {
			log::debug!("parsing succeeded.");
			let mut context = treeldr::Context::new(args.base_iri);
			let mut env = treeldr::compile::Environment::new(&mut context);
			match doc.compile(&mut env) {
				Ok(_) => {
					log::debug!("compilation succeeded.");
				},
				Err(e) => {
					log::error!("compilation error");
					let diagnostic = e.with_context(&context).diagnostic();
					let writer = StandardStream::stderr(ColorChoice::Always);
					let config = codespan_reporting::term::Config::default();
					term::emit(&mut writer.lock(), &config, &files, &diagnostic)
						.expect("diagnostic failed");
					std::process::exit(1);
				}
			}
		}
		Err(e) => {
			log::error!("parsing error");
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
