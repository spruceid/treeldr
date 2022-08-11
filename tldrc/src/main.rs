use clap::Parser;
use codespan_reporting::term::{
	self,
	termcolor::{ColorChoice, StandardStream},
};
use std::path::PathBuf;
use treeldr_load as load;
use treeldr_syntax as syntax;

mod source;
use source::Source;

type BuildContext = treeldr_build::Context<load::FileId, syntax::build::Descriptions>;

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
	Rdf {
		#[clap(short = 's', long = "standard-vocab")]
		include_standard_vocabulary: bool,
	},

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

	let mut files = load::Files::<Source>::new();
	let mut documents = Vec::new();

	documents.push(load_built_in(
		&mut files,
		Source::Xsd,
		include_str!("../../schema/xsd.tldr"),
	));

	for filename in args.filenames {
		match load::Document::load(&mut files, &filename) {
			Ok((document, _)) => documents.push(document),
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

	match load::build_all(&mut vocabulary, &mut build_context, documents) {
		Ok(model) =>
		{
			#[allow(unused_variables)]
			match args.command {
				Some(Command::Rdf {
					include_standard_vocabulary,
				}) => {
					use treeldr::vocab::RdfDisplay;
					let mut quads = Vec::new();
					model.to_rdf_with(
						&mut vocabulary,
						&mut quads,
						treeldr::to_rdf::Options {
							ignore_standard_vocabulary: !include_standard_vocabulary,
						},
					);
					for quad in quads {
						println!("{} .", quad.rdf_display(&vocabulary))
					}
				}
				#[cfg(feature = "json-schema")]
				Some(Command::JsonSchema(command)) => command.execute(&vocabulary, &model),
				#[cfg(feature = "json-ld-context")]
				Some(Command::JsonLdContext(command)) => command.execute(&vocabulary, &model),
				_ => (),
			}
		}
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

fn load_built_in(files: &mut load::Files<Source>, source: Source, content: &str) -> load::Document {
	let file_id = files.load_content(
		source,
		None,
		Some(load::MimeType::TreeLdr),
		content.to_string(),
	);
	load::Document::TreeLdr(Box::new(load::import_treeldr(files, file_id)))
}
