use clap::{self, load_yaml};
use codespan_reporting::{
	term::{
		self,
		termcolor::{ColorChoice, StandardStream},
	},
};
use iref::IriBuf;
use std::{convert::Infallible, fs, io};
use treeldr::{source, syntax, syntax::Parse, Compile, error::Diagnose};

fn main() -> io::Result<()> {
	// Parse options.
	let yaml = load_yaml!("treeldr.yml");
	let matches = clap::App::from_yaml(yaml).get_matches();

	// Init logger.
	let verbosity = matches.occurrences_of("verbose") as usize;
	stderrlog::new().verbosity(verbosity).init().unwrap();

	let base_iri = IriBuf::new(matches.value_of("BASE_IRI").unwrap()).expect("invalid base IRI");

	let filename = matches.value_of("FILE").unwrap();
	let content = fs::read_to_string(filename)?;

	let mut files = source::Files::new();
	let (source_id, file) = files.add(source::Path::Local(filename.into()), content);

	let mut lexer =
		syntax::Lexer::<Infallible, _>::new(source_id, file.buffer().chars().map(Result::Ok))
			.peekable();

	// for token in lexer {
	// 	eprintln!("token: {:?}", token)
	// }

	match syntax::Document::parse(source_id, &mut lexer, 0) {
		Ok(doc) => {
			log::info!("parsing succeeded.");
			let mut context = treeldr::Context::new(base_iri);
			match doc.compile(&mut context) {
				Ok(_) => {
					log::info!("compilation succeeded.");
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
