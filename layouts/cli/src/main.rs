use codespan_reporting::{
	diagnostic::{Diagnostic, Label},
	files::SimpleFiles,
	term::{
		self,
		termcolor::{ColorChoice, StandardStream},
	},
};
use std::{fs, path::PathBuf, process::ExitCode};

#[derive(clap::Parser)]
#[clap(name="tldr", author, version, about, long_about = None)]
struct Args {
	/// Input files.
	filenames: Vec<PathBuf>,

	/// Sets the level of verbosity.
	#[clap(short, long = "verbose", action = clap::ArgAction::Count)]
	verbosity: u8,
}

fn main() -> ExitCode {
	// Parse options.
	let args: Args = clap::Parser::parse();

	// Initialize logger.
	stderrlog::new()
		.verbosity(args.verbosity as usize)
		.init()
		.unwrap();

	let mut files = SimpleFiles::new();

	for filename in args.filenames {
		match fs::read_to_string(&filename) {
			Ok(content) => {
				let file_id = files.add(filename.to_string_lossy().into_owned(), content);
				if !load_file(&files, file_id) {
					return ExitCode::FAILURE;
				}
			}
			Err(e) => {
				eprintln!("Unable to read file `{}`: {e}", filename.display());
				return ExitCode::FAILURE;
			}
		}
	}

	ExitCode::SUCCESS
}

fn load_file(files: &SimpleFiles<String, String>, file_id: usize) -> bool {
	use json_syntax::{Parse, TryFromJsonSyntax};

	match json_syntax::Value::parse_str(files.get(file_id).unwrap().source().as_str()) {
		Ok((json, code_map)) => {
			match treeldr_layouts::abs::syntax::Layout::try_from_json_syntax(&json, &code_map) {
				Ok(_layout) => true,
				Err(e) => {
					let span = code_map.get(e.position()).unwrap().span;
					let diagnostic = Diagnostic::error()
						.with_message("Layout syntax error")
						.with_labels(vec![
							Label::primary(file_id, span).with_message(e.to_string())
						])
						.with_notes(e.hints().into_iter().map(|h| h.to_string()).collect());

					let writer = StandardStream::stderr(ColorChoice::Always);
					let config = codespan_reporting::term::Config::default();
					term::emit(&mut writer.lock(), &config, files, &diagnostic).unwrap();
					false
				}
			}
		}
		Err(e) => {
			let diagnostic = Diagnostic::error()
				.with_message("JSON error")
				.with_labels(vec![
					Label::primary(file_id, e.span()).with_message(e.to_string())
				]);
			let writer = StandardStream::stderr(ColorChoice::Always);
			let config = codespan_reporting::term::Config::default();
			term::emit(&mut writer.lock(), &config, files, &diagnostic).unwrap();
			false
		}
	}
}
