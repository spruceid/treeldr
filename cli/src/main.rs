use clap::Parser;
use codespan_reporting::term::{
	self,
	termcolor::{ColorChoice, StandardStream},
};
use iref::IriBuf;
use std::{convert::Infallible, fmt, fs, io, path::PathBuf};
use treeldr::{error::Diagnose, source, syntax, syntax::Parse, Build};

#[derive(Parser)]
#[clap(name="treeldr", author, version, about, long_about = None)]
struct Args {
	/// Base IRI.
	base_iri: IriBuf,

	/// Input TreeLDR file.
	#[clap(name = "FILE")]
	filename: PathBuf,

	#[clap(short = 'M', parse(try_from_str = parse_mount_point), multiple_occurrences(true))]
	/// Mount a filename to an IRI with the syntax `IRI=FILENAME`.
	mounts: Vec<(IriBuf, PathBuf)>,

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

			let mut env = treeldr::build::Environment::new(&mut model);
			match doc.build(&mut env) {
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

#[derive(Debug)]
pub enum MountPointError {
	InvalidPath(String),
	MissingEqual,
	InvalidIRI(String)
}

impl fmt::Display for MountPointError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::InvalidPath(s) => write!(f, "invalid path `{}`", s),
			Self::MissingEqual => "missing `=` between IRI and path".fmt(f),
			Self::InvalidIRI(s) => write!(f, "invalid IRI `{}`", s)
		}
	}
}

impl std::error::Error for MountPointError {}

fn parse_mount_point(s: &str) -> Result<(IriBuf, PathBuf), MountPointError> {
	let pos = s.find('=').ok_or(MountPointError::MissingEqual)?;
	let iri = s[..pos].parse().map_err(|_| MountPointError::InvalidIRI(s[..pos].to_string()))?;
	let filename = s[pos+1..].parse().map_err(|_| MountPointError::InvalidPath(s[pos+1..].to_string()))?;
	Ok((iri, filename))
}