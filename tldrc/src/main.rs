use clap::Parser;
use codespan_reporting::term::{
	self,
	termcolor::{ColorChoice, StandardStream},
};
use contextual::WithContext;
use std::path::PathBuf;
use treeldr::to_rdf::ToRdf;
use treeldr_load as load;

mod source;
use source::Source;

type BuildContext = treeldr_build::Context<load::Metadata>;

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

#[async_std::main]
async fn main() {
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
	let mut vocabulary = rdf_types::IndexVocabulary::new();
	let mut generator = rdf_types::generator::Blank::new();
	let mut build_context = BuildContext::new();

	match load::build_all(
		&mut vocabulary,
		&mut generator,
		&mut build_context,
		documents,
	) {
		Ok(model) =>
		{
			#[allow(unused_variables)]
			match args.command {
				Some(Command::Rdf {
					include_standard_vocabulary,
				}) => {
					let mut quads = Vec::new();
					model.to_rdf_with(
						&mut vocabulary,
						&mut generator,
						&mut quads,
						treeldr::to_rdf::Options {
							ignore_standard_vocabulary: !include_standard_vocabulary,
						},
					);

					quads.sort();

					for quad in quads {
						println!("{} .", quad.with(&vocabulary))
					}
				}
				#[cfg(feature = "json-schema")]
				Some(Command::JsonSchema(command)) => command.execute(&vocabulary, &model),
				#[cfg(feature = "json-ld-context")]
				Some(Command::JsonLdContext(command)) => {
					command.execute(&mut vocabulary, &mut files, &model).await
				}
				_ => (),
			}
		}
		Err(e) => {
			let diagnostic = e.with(&vocabulary).diagnostic();
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
