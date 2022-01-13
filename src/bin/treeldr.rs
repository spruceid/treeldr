use clap::{self, load_yaml};
use codespan_reporting::{
	diagnostic::Diagnostic,
	term::{
		self,
		termcolor::{ColorChoice, StandardStream},
	},
};
use std::{convert::Infallible, fs, io};
use treeldr::{source, syntax, syntax::Parse, Error};

fn main() -> io::Result<()> {
	// Parse options.
	let yaml = load_yaml!("treeldr.yml");
	let matches = clap::App::from_yaml(yaml).get_matches();

	// Init logger.
	let verbosity = matches.occurrences_of("verbose") as usize;
	stderrlog::new().verbosity(verbosity).init().unwrap();

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
		Ok(_doc) => {
			println!("parsing succeeded.");
		}
		Err(e) => {
			let diagnostic = Diagnostic::error()
				.with_message(e.message())
				.with_labels(e.labels())
				.with_notes(e.notes());
			let writer = StandardStream::stderr(ColorChoice::Always);
			let config = codespan_reporting::term::Config::default();
			term::emit(&mut writer.lock(), &config, &files, &diagnostic)
				.expect("diagnostic failed");
		}
	}

	Ok(())
}
