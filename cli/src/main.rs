use codespan_reporting::{
	diagnostic::{Diagnostic, Label},
	files::SimpleFiles,
	term::{
		self,
		termcolor::{ColorChoice, StandardStream},
	},
};
use std::{
	fs,
	io::{self, BufReader, BufWriter, Write},
	path::PathBuf,
	process::ExitCode,
};
use treeldr::{
	build::TypeResolutionError,
	display::DisplaySignature,
	eval::{EvalError, MissingData},
	module::ItemRef,
	package::CompileError,
	rdf::{
		dataset::{BTreeDataset, IndexedBTreeDataset},
		Rdf,
	},
	syntax::Spanned,
	value::ValueParseError,
	Registry as _, Value,
};

mod format;
use format::{RdfFormat, TreeFormat};

type PackageId = treeldr::registry::PreludeOrTarget;

type Registry = treeldr::registry::PreludeAndTarget;

#[derive(clap::Parser)]
#[clap(name="tldr", author, version, about, long_about = None, subcommand_precedence_over_arg = true)]
struct Args {
	/// Input file.
	input: PathBuf,

	/// Sets the level of verbosity.
	#[arg(short, long = "verbose", action = clap::ArgAction::Count, global = true)]
	verbosity: u8,

	#[command(subcommand)]
	command: Option<Command>,
}

#[derive(clap::Subcommand)]
enum Command {
	Compile,

	/// Call a function.
	Call {
		/// Format of the input arguments that are read from the standard input.
		#[arg(short = 'i', long = "input", value_parser = TreeFormat::parser(), default_value = "tron")]
		input_format: TreeFormat,

		/// Format of the output values.
		#[arg(short = 'o', long = "output", value_parser = TreeFormat::parser(), default_value = "tron")]
		output_format: TreeFormat,

		/// Format of the input RDF dataset.
		#[arg(short = 'n', long = "rdf-input", value_names = ["FILENAME", "FORMAT"], num_args(1..=2))]
		rdf_input: Option<Vec<String>>,

		/// Format of the output RDF dataset.
		#[arg(short = 'u', long = "rdf-output", value_names = ["FILENAME", "FORMAT"], num_args(1..=2))]
		rdf_output: Option<Vec<String>>,

		/// Function arguments.
		///
		/// If the inverse function is called, this is expected to be
		/// the function's const arguments and output.
		#[arg(value_parser = value_parser)]
		args: Vec<Value>,

		/// Call the inverse function.
		#[arg(short = 'r', long = "reverse")]
		reverse: bool,

		/// Pretty print the output.
		#[arg(short, long)]
		pretty: bool,
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

	let mut rdf = Rdf::default();
	let mut registry = Registry::new();

	match args.run(&mut rdf, &mut registry, &mut files) {
		Ok(()) => ExitCode::SUCCESS,
		Err(e) => {
			let diagnostic = e.into_diagnostic(&registry);
			let writer = StandardStream::stderr(ColorChoice::Always);
			let config = codespan_reporting::term::Config::default();
			term::emit(&mut writer.lock(), &config, &files, &diagnostic).unwrap();
			ExitCode::FAILURE
		}
	}
}

impl Args {
	fn run(
		self,
		rdf: &mut Rdf,
		registry: &mut Registry,
		files: &mut SimpleFiles<String, String>,
	) -> Result<(), Error> {
		let content = fs::read_to_string(&self.input).map_err(Error::IO)?;
		let file_id = files.add(self.input.to_string_lossy().into_owned(), content);

		registry.target = Some(
			treeldr::Module::compile(
				rdf,
				registry,
				PackageId::Target,
				files.get(file_id).unwrap().source().as_str(),
			)
			.map_err(|e| Error::Compile(file_id, e))?,
		);

		match self.command {
			Some(command) => command.run(rdf, &registry),
			None => {
				println!(
					"{}",
					registry
						.target
						.as_ref()
						.unwrap()
						.root()
						.display_signature(&registry)
				);
				Ok(())
			}
		}
	}
}

enum RdfInput {
	Stdin,
	File(PathBuf),
}

impl RdfInput {
	pub fn load(self, format: RdfFormat) -> Result<BTreeDataset, Error> {
		match self {
			Self::Stdin => {
				let stdin = BufReader::new(io::stdin());
				format.load(stdin).map_err(Error::LoadRdf)
			}
			Self::File(path) => {
				let file = BufReader::new(fs::File::open(path)?);
				format.load(file).map_err(Error::LoadRdf)
			}
		}
	}
}

enum RdfOutput {
	Stdout,
	File(PathBuf),
}

impl RdfOutput {
	pub fn save(self, dataset: IndexedBTreeDataset, format: RdfFormat) -> Result<(), Error> {
		match self {
			Self::Stdout => {
				let mut stdout = BufWriter::new(io::stdout());
				stdout.write(b"\n")?;
				format.write(dataset, stdout).map_err(Into::into)
			}
			Self::File(path) => {
				let file = fs::File::create(path)?;
				format.write(dataset, file).map_err(Into::into)
			}
		}
	}
}

impl Command {
	fn run(self, rdf: &mut Rdf, registry: &Registry) -> Result<(), Error> {
		match self {
			Self::Compile => {
				let package = registry.target.as_ref().unwrap();
				let config = ron::ser::PrettyConfig::new();
				println!(
					"{}",
					ron::ser::to_string_pretty(&package.with_rdf(rdf), config).unwrap()
				);
				Ok(())
			}
			Self::Call {
				input_format,
				output_format,
				rdf_input,
				rdf_output,
				mut args,
				reverse,
				pretty,
			} => {
				let module = registry.target.as_ref().unwrap().root();

				let name = "_";

				let f_ref = match module.get_by_name(name) {
					Some(ItemRef::Function(f_ref)) => f_ref,
					Some(_) => return Err(Error::NotAFunction(name.to_owned())),
					None => return Err(Error::UndefinedFunction(name.to_owned())),
				};

				let function = registry.get_function(f_ref).unwrap();

				let expected_arg_count = if reverse {
					function.signature.const_args.len() + 1
				} else {
					function.signature.arity() as usize
				};

				while args.len() < expected_arg_count {
					println!("expecting one argument");
					let stdin = BufReader::new(io::stdin());
					args.push(input_format.load(stdin)?)
				}

				let mut dataset = match rdf_input {
					Some(options) => {
						let mut options = options.into_iter();
						let filename = options.next().unwrap();

						let input = if filename == "-" {
							RdfInput::Stdin
						} else {
							RdfInput::File(filename.into())
						};

						let format = options
							.next()
							.map(|name| name.parse::<RdfFormat>())
							.transpose()?
							.unwrap_or_default();

						input.load(format)?.into_indexed()
					}
					None => IndexedBTreeDataset::new(),
				};

				if reverse {
					let mut args = args.into_iter();

					let const_args: Vec<_> = function
						.signature
						.const_args
						.iter()
						.map(|_| args.next().unwrap())
						.collect();

					let output = args.next().unwrap();

					let inputs =
						function.call_inverse(registry, rdf, &mut dataset, &const_args, &output)?;

					let mut out = io::stdout();

					for (ty, input) in function.signature.args.iter().copied().zip(inputs) {
						let typed_input = input.into_typed(registry, ty).unwrap();

						output_format.write(registry, &typed_input, pretty, &mut out)?;
						out.write(b"\n")?;
					}

					out.flush()?;
				} else {
					let output = function.call(registry, rdf, &mut dataset, &args)?;

					let typed_output = output
						.into_typed(registry, function.signature.return_type)
						.unwrap();

					output_format.write(registry, &typed_output, pretty, io::stdout())?;
				}

				if let Some(options) = rdf_output {
					let mut options = options.into_iter();
					let filename = options.next().unwrap();

					let output = if filename == "-" {
						RdfOutput::Stdout
					} else {
						RdfOutput::File(filename.into())
					};

					let format = options
						.next()
						.map(|name| name.parse::<RdfFormat>())
						.transpose()?
						.unwrap_or_default();

					output.save(dataset, format)?
				}

				Ok(())
			}
		}
	}
}

fn value_parser(s: &str) -> Result<Value, ValueParseError> {
	s.parse()
}

#[derive(Debug, thiserror::Error)]
enum Error {
	#[error(transparent)]
	IO(#[from] io::Error),

	#[error("{1}")]
	Compile(usize, CompileError<PackageId>),

	#[error("undefined function `{0}`")]
	UndefinedFunction(String),

	#[error("`{0}` is not a function")]
	NotAFunction(String),

	#[error(transparent)]
	InvalidRdfFormat(#[from] format::rdf::InvalidRdfFormat),

	#[error(transparent)]
	LoadRdf(#[from] format::rdf::LoadError),

	#[error(transparent)]
	LoadTree(#[from] format::tree::LoadError),

	#[error(transparent)]
	Eval(#[from] treeldr::eval::EvalError),

	#[error(transparent)]
	WriteTree(#[from] format::tree::WriteError),
}

impl Error {
	fn into_diagnostic(self, registry: &Registry) -> Diagnostic<usize> {
		match self {
			Self::Compile(file_id, e) => match e {
				CompileError::Typing(TypeResolutionError::Subtyping {
					sub_type,
					super_type,
				}) => Diagnostic::error()
					.with_notes(vec![
						format!(
							"type                {}",
							sub_type.display_signature(registry)
						),
						format!(
							"is not a subtype of {}",
							super_type.display_signature(registry)
						),
					])
					.with_message(e.to_string()),
				CompileError::Typing(e) => match e {
					TypeResolutionError::Unification(a, b) => {
						let mut labels = Vec::new();

						if let Some(span) = a.span {
							labels.push(Label::primary(file_id, span));
						}

						if let Some(span) = b.span {
							labels.push(Label::primary(file_id, span));
						}

						Diagnostic::error()
							.with_labels(labels)
							.with_message("type mismatch")
					}
					TypeResolutionError::Definition(span) => {
						let mut labels = Vec::new();

						if let Some(span) = span {
							labels.push(Label::primary(file_id, span));
						}

						Diagnostic::error()
							.with_labels(labels)
							.with_message("type annotation needed")
					}
					e => Diagnostic::error().with_message(e.to_string()),
				},
				CompileError::Build(e) => Diagnostic::error()
					.with_labels(vec![
						Label::primary(file_id, e.span()).with_message(e.to_string())
					])
					.with_message("build error"),
				CompileError::Parse(e) => Diagnostic::error()
					.with_labels(vec![
						Label::primary(file_id, e.span()).with_message(e.to_string())
					])
					.with_message("invalid syntax"),
			},
			Self::Eval(EvalError::MissingData(MissingData::Empty(dataset))) => Diagnostic::error()
				.with_message("missing required RDF quads")
				.with_notes(dataset.iter().map(|quad| quad.to_string()).collect()),
			e => Diagnostic::error().with_message(e.to_string()),
		}
	}
}
