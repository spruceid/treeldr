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
	io::{self, BufReader},
	path::PathBuf,
	process::ExitCode,
};
use treeldr::{
	build::TypeResolutionError,
	display::DisplaySignature,
	eval::{EvalError, MissingData},
	module::{CompileError, ItemRef},
	rdf::{dataset::BTreeDataset, Rdf},
	syntax::Spanned,
	value::ValueParseError,
	Environment, ModuleRef, Registry, Value,
};

mod format;
use format::{RDFFormat, TreeFormat};

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
	/// Serializes an RDF dataset into a tree value.
	Hydrate {
		/// Format of the input RDF dataset.
		#[arg(short, long = "input", value_parser = RDFFormat::parser(), default_value = "n-quads")]
		input_format: RDFFormat,

		/// Format of the output tree value.
		#[arg(short, long = "output", value_parser = TreeFormat::parser(), default_value = "json")]
		output_format: TreeFormat,

		/// Function arguments.
		#[arg(value_parser = value_parser)]
		args: Vec<Value>,

		/// Pretty print the output.
		#[arg(short, long)]
		pretty: bool,
	},

	/// Deserializes a tree value into an RDF dataset.
	Dehydrate {
		/// Format of the input tree value.
		#[arg(short, long = "input", value_parser = TreeFormat::parser(), default_value = "json")]
		input_format: TreeFormat,

		/// Format of the output RDF dataset.
		#[arg(short, long = "output", value_parser = RDFFormat::parser(), default_value = "n-quads")]
		output_format: RDFFormat,
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
	let prelude_ref = registry.add_prelude();
	let env = Environment::new(Some(prelude_ref));

	match args.run(&mut rdf, &mut registry, &env, &mut files) {
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
		env: &Environment,
		files: &mut SimpleFiles<String, String>,
	) -> Result<(), Error> {
		let content = fs::read_to_string(&self.input).map_err(Error::IO)?;
		let file_id = files.add(self.input.to_string_lossy().into_owned(), content);

		let module_ref = treeldr::Module::compile(
			rdf,
			registry,
			env,
			files.get(file_id).unwrap().source().as_str(),
		)
		.map_err(|e| Error::Compile(file_id, e))?;

		match self.command {
			Some(command) => command.run(rdf, registry, module_ref),
			None => {
				let module = registry.get_module(module_ref).unwrap();
				println!("{}", module.display_signature(registry));
				Ok(())
			}
		}
	}
}

impl Command {
	fn run(self, rdf: &mut Rdf, registry: &Registry, module_ref: ModuleRef) -> Result<(), Error> {
		match self {
			Self::Hydrate {
				input_format,
				output_format,
				args,
				pretty,
			} => {
				let module = registry.get_module(module_ref).unwrap();

				let name = "_";

				let f_ref = match module.get_by_name(name) {
					Some(ItemRef::Function(f_ref)) => f_ref,
					Some(_) => return Err(Error::NotAFunction(name.to_owned())),
					None => return Err(Error::UndefinedFunction(name.to_owned())),
				};

				let function = registry.get_function(f_ref).unwrap();

				let stdin = BufReader::new(io::stdin());
				let dataset = input_format
					.load(stdin)
					.map_err(Error::LoadRdf)?
					.into_indexed();

				let output = function.call(registry, rdf, &dataset, &args)?;

				let typed_output = output
					.into_typed(registry, function.signature.return_type)
					.unwrap();

				output_format
					.write(registry, &typed_output, pretty, io::stdout())
					.map_err(Into::into)
			}
			Self::Dehydrate {
				input_format,
				output_format,
			} => {
				let module = registry.get_module(module_ref).unwrap();

				let name = "_";

				let f_ref = match module.get_by_name(name) {
					Some(ItemRef::Function(f_ref)) => f_ref,
					Some(_) => return Err(Error::NotAFunction(name.to_owned())),
					None => return Err(Error::UndefinedFunction(name.to_owned())),
				};

				let function = registry.get_function(f_ref).unwrap();

				let stdin = BufReader::new(io::stdin());
				let tree = input_format.load(stdin).map_err(Error::LoadTree)?;

				let mut dataset = BTreeDataset::new();
				let _ = function.call_inverse(registry, rdf, &mut dataset, &[], &tree)?;

				let dataset: BTreeDataset = dataset
					.into_iter()
					.map(|quad| quad.map(|t| rdf.interpretation.map_term(t)))
					.collect();

				output_format
					.write(dataset, io::stdout())
					.map_err(Into::into)
			}
		}
	}
}

fn value_parser(s: &str) -> Result<Value, ValueParseError> {
	s.parse()
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error(transparent)]
	IO(#[from] io::Error),

	#[error("{1}")]
	Compile(usize, CompileError),

	#[error("undefined function `{0}`")]
	UndefinedFunction(String),

	#[error("`{0}` is not a function")]
	NotAFunction(String),

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
				CompileError::Typing(TypeResolutionError::Unification(a, b)) => {
					let mut labels = Vec::new();

					if let Some(span) = a {
						labels.push(Label::primary(file_id, span));
					}

					if let Some(span) = b {
						labels.push(Label::primary(file_id, span));
					}

					Diagnostic::error()
						.with_labels(labels)
						.with_message("type mismatch")
				}
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
				e => Diagnostic::error().with_message(e.to_string()),
			},
			Self::Eval(EvalError::MissingData(MissingData::Empty(dataset))) => Diagnostic::error()
				.with_message("no matching data")
				.with_notes(dataset.iter().map(|quad| quad.to_string()).collect()),
			e => Diagnostic::error().with_message(e.to_string()),
		}
	}
}
