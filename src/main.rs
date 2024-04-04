use codespan_reporting::{
	diagnostic::{Diagnostic, Label},
	files::SimpleFiles,
	term::{
		self,
		termcolor::{ColorChoice, StandardStream},
	},
};
use rdf_types::Term;
use std::{
	fs,
	io::{self, BufReader},
	path::PathBuf,
	process::ExitCode,
};
use treeldr_layouts::{layout::LayoutType, Layouts, Ref};

mod format;
use format::{RDFFormat, TreeFormat};
mod rdf;

#[derive(clap::Parser)]
#[clap(name="tldr", author, version, about, long_about = None, subcommand_precedence_over_arg = true)]
struct Args {
	/// Input layout files.
	layouts: Vec<PathBuf>,

	/// More layout files.
	///
	/// Those layouts will not contribute to the search for a default layout.
	#[arg(short, long)]
	include: Vec<PathBuf>,

	/// Sets the level of verbosity.
	#[arg(short, long = "verbose", action = clap::ArgAction::Count, global = true)]
	verbosity: u8,

	#[command(subcommand)]
	command: Option<Command>,
}

#[derive(clap::Subcommand)]
enum Command {
	/// Serializes an RDF dataset into a tree value.
	Hydrate {
		/// Format of the input RDF dataset.
		#[arg(short, long, default_value = "n-quads")]
		input: RDFFormat,

		/// Format of the output tree value.
		#[arg(short, long, default_value = "json")]
		output: TreeFormat,

		/// Serializing layout.
		///
		/// If only one layout file is given with a single top-level layout,
		/// this layout will be selected by TreeLDR by default.
		/// Otherwise, this argument is required.
		#[arg(short, long, value_parser = rdf::parse_term)]
		layout: Option<Term>,

		/// Entry points to the dataset, provided as layout input.
		#[arg(value_parser = rdf::parse_term)]
		subjects: Vec<Term>,

		/// Pretty print the output.
		#[arg(short, long)]
		pretty: bool,
	},

	/// Deserializes a tree value into an RDF dataset.
	Dehydrate {
		/// Format of the input tree value.
		#[clap(short, long, default_value = "json")]
		input: TreeFormat,

		/// Format of the output RDF dataset.
		#[clap(short, long, default_value = "n-quads")]
		output: RDFFormat,

		/// Deserializing layout.
		///
		/// If only one layout file is given with a single top-level layout,
		/// this layout will be selected by TreeLDR by default.
		/// Otherwise, this argument is required.
		#[arg(short, long, value_parser = rdf::parse_term)]
		layout: Option<Term>,
	},
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

	match run(&mut files, args) {
		Ok(()) => ExitCode::SUCCESS,
		Err(e) => {
			let diagnostic = e.into_diagnostic();
			let writer = StandardStream::stderr(ColorChoice::Always);
			let config = codespan_reporting::term::Config::default();
			term::emit(&mut writer.lock(), &config, &files, &diagnostic).unwrap();
			ExitCode::FAILURE
		}
	}
}

enum DefaultLayoutRef {
	None,
	Some(Ref<LayoutType>),
	Ambiguous,
}

impl DefaultLayoutRef {
	fn set(&mut self, layout_ref: Ref<LayoutType>) {
		match self {
			Self::None => *self = Self::Some(layout_ref),
			Self::Some(_) => *self = Self::Ambiguous,
			_ => (),
		}
	}

	fn get(self, preferred_layout: Option<Term>) -> Result<Ref<LayoutType>, Error> {
		match preferred_layout {
			Some(term) => Ok(Ref::new(term)),
			None => match self {
				Self::None => Err(Error::NoDefaultLayout),
				Self::Some(layout_ref) => Ok(layout_ref),
				Self::Ambiguous => Err(Error::AmbiguousDefaultLayout),
			},
		}
	}
}

fn run(files: &mut SimpleFiles<String, String>, args: Args) -> Result<(), Error> {
	let mut layouts = Layouts::new();
	let mut default_layout = DefaultLayoutRef::None;
	for filename in args.layouts {
		let content = fs::read_to_string(&filename).map_err(Error::IO)?;
		let file_id = files.add(filename.to_string_lossy().into_owned(), content);
		let layout_ref = load_layout(files, file_id, &mut layouts)?;
		default_layout.set(layout_ref);
	}

	for filename in args.include {
		let content = fs::read_to_string(&filename).map_err(Error::IO)?;
		let file_id = files.add(filename.to_string_lossy().into_owned(), content);
		load_layout(files, file_id, &mut layouts)?;
	}

	match args.command {
		None => Ok(()),
		Some(command) => command.run(layouts, default_layout),
	}
}

impl Command {
	fn run(self, layouts: Layouts, default_layout: DefaultLayoutRef) -> Result<(), Error> {
		match self {
			Self::Hydrate {
				input,
				output,
				layout,
				subjects,
				pretty,
			} => {
				let layout_ref = default_layout.get(layout)?;
				let stdin = BufReader::new(io::stdin());
				let input = input.load(stdin).map_err(Error::LoadRdf)?.into_indexed();
				let output_data =
					treeldr_layouts::distill::hydrate(&layouts, &input, &layout_ref, &subjects)
						.map_err(Error::Hydrate)?;
				output
					.write(output_data, pretty, io::stdout())
					.map_err(Error::CreateTree)
			}
			Self::Dehydrate {
				input,
				output,
				layout,
			} => {
				let layout_ref = default_layout.get(layout)?;
				let stdin = BufReader::new(io::stdin());
				let input = input.load(stdin).map_err(Error::LoadTree)?;
				let (output_data, _) = treeldr_layouts::distill::dehydrate(
					&layouts,
					&input,
					&layout_ref,
					Default::default(),
				)
				.map_err(Error::Dehydrate)?;
				output.write(output_data, io::stdout()).map_err(Error::IO)
			}
		}
	}
}

enum Error {
	IO(io::Error),
	JsonSyntax(usize, json_syntax::parse::Error),
	LayoutSyntax(
		usize,
		json_syntax::CodeMap,
		treeldr_layouts::abs::syntax::Error,
	),
	LayoutBuild(usize, treeldr_layouts::abs::syntax::BuildError),
	NoDefaultLayout,
	AmbiguousDefaultLayout,
	LoadRdf(format::rdf::LoadError),
	LoadTree(format::tree::LoadError),
	Hydrate(treeldr_layouts::distill::hy::Error),
	Dehydrate(treeldr_layouts::distill::de::Error),
	CreateTree(format::tree::WriteError),
}

impl Error {
	fn into_diagnostic(self) -> Diagnostic<usize> {
		match self {
			Self::IO(e) => Diagnostic::error().with_message(e.to_string()),
			Self::JsonSyntax(file_id, e) => Diagnostic::error()
				.with_message("JSON error")
				.with_labels(vec![
					Label::primary(file_id, e.span()).with_message(e.to_string())
				]),
			Self::LayoutSyntax(file_id, code_map, e) => {
				let span = code_map.get(e.position()).unwrap().span;

				let mut labels = vec![Label::primary(file_id, span).with_message(e.to_string())];

				let mut notes = Vec::new();
				for hint in e.hints() {
					match hint.position() {
						Some(i) => {
							let span = code_map.get(i).unwrap().span;
							labels.push(
								Label::secondary(file_id, span).with_message(hint.to_string()),
							)
						}
						None => notes.push(hint.to_string()),
					}
				}

				Diagnostic::error()
					.with_message("Layout syntax error")
					.with_labels(labels)
					.with_notes(notes)
			}
			Self::LayoutBuild(_file_id, e) => Diagnostic::error().with_message(e.to_string()),
			Self::NoDefaultLayout => Diagnostic::error()
				.with_message("no default layout")
				.with_notes(vec![
					"use the `--layout` option to specify what layout to use".to_owned(),
				]),
			Self::AmbiguousDefaultLayout => Diagnostic::error()
				.with_message("ambiguous layout")
				.with_notes(vec![
					"use the `--layout` option to specify what layout to use".to_owned(),
				]),
			Self::LoadRdf(e) => Diagnostic::error().with_message(e.to_string()),
			Self::LoadTree(e) => Diagnostic::error().with_message(e.to_string()),
			Self::Hydrate(e) => Diagnostic::error().with_message(e.to_string()),
			Self::Dehydrate(e) => Diagnostic::error().with_message(e.to_string()),
			Self::CreateTree(e) => Diagnostic::error().with_message(e.to_string()),
		}
	}
}

/// Loads a layout file.
fn load_layout(
	files: &SimpleFiles<String, String>,
	file_id: usize,
	layouts: &mut Layouts,
) -> Result<Ref<LayoutType>, Error> {
	use json_syntax::{Parse, TryFromJson};

	let mut builder = treeldr_layouts::abs::Builder::new();
	match json_syntax::Value::parse_str(files.get(file_id).unwrap().source().as_str()) {
		Ok((json, code_map)) => {
			match treeldr_layouts::abs::syntax::Layout::try_from_json(&json, &code_map) {
				Ok(layout) => match layout.build(&mut builder) {
					Ok(layout_ref) => {
						let new_layouts = builder.build();

						for (id, layout) in new_layouts {
							let (_, old_layout) = layouts.insert(id.into_id(), layout);
							if old_layout.is_some() {
								return Err(Error::LayoutBuild(
									file_id,
									treeldr_layouts::abs::syntax::BuildError::LayoutRedefinition,
								));
							}
						}

						Ok(layout_ref)
					}
					Err(e) => Err(Error::LayoutBuild(file_id, e)),
				},
				Err(e) => Err(Error::LayoutSyntax(file_id, code_map, e)),
			}
		}
		Err(e) => Err(Error::JsonSyntax(file_id, e)),
	}
}
