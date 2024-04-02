use iref::IriBuf;
use rdf_types::Term;
use std::{fs, path::PathBuf, process::ExitCode};
use treeldr_layouts::{abs, distill::RdfContext, layout::LayoutType, Ref};

#[derive(clap::Parser)]
#[clap(name="tldr-rs", author, version, about, long_about = None)]
struct Args {
	/// Input files.
	filenames: Vec<PathBuf>,

	/// Layout to generate.
	#[clap(long, short)]
	layout: Option<IriBuf>,

	/// Sets the level of verbosity.
	#[clap(short, long = "verbose", action = clap::ArgAction::Count)]
	verbosity: u8,
}

enum DefaultLayoutRef {
	Unknown,
	Some(Ref<LayoutType>),
	None,
}

impl DefaultLayoutRef {
	pub fn set(&mut self, layout_ref: Ref<LayoutType>) {
		match self {
			Self::Unknown => *self = Self::Some(layout_ref),
			Self::Some(_) => *self = Self::None,
			Self::None => (),
		}
	}
}

fn main() -> ExitCode {
	// Parse options.
	let args: Args = clap::Parser::parse();

	// Initialize logger.
	stderrlog::new()
		.verbosity(args.verbosity as usize)
		.init()
		.unwrap();

	// Initialize the layout builder.
	let mut builder = abs::Builder::new();

	let mut default_layout_ref = DefaultLayoutRef::Unknown;

	for filename in args.filenames {
		let content = fs::read_to_string(filename).unwrap();

		match serde_json::from_str::<abs::syntax::Layout>(&content) {
			Ok(abstract_layout) => match abstract_layout.build(&mut builder) {
				Ok(layout_ref) => default_layout_ref.set(layout_ref),
				Err(e) => {
					log::error!("compile error: {e}");
					return ExitCode::FAILURE;
				}
			},
			Err(e) => {
				log::error!("parse error: {e}")
			}
		}
	}

	let layouts = builder.build();

	let layout_ref = match args.layout {
		Some(iri) => {
			let term = Term::iri(iri);
			if layouts.layout(&term).is_some() {
				Ref::new(term)
			} else {
				log::error!("unknown layout {term}");
				return ExitCode::FAILURE;
			}
		}
		None => match default_layout_ref {
			DefaultLayoutRef::Some(layout_ref) => layout_ref,
			_ => {
				log::error!("missing layout");
				return ExitCode::FAILURE;
			}
		},
	};

	let gen_options = treeldr_gen_rust::Options::new();

	let result =
		treeldr_gen_rust::generate(RdfContext::default(), &layouts, &layout_ref, &gen_options);

	match result {
		Ok(r) => {
			println!("{r}");
			ExitCode::SUCCESS
		}
		Err(e) => {
			log::error!("parse error: {e}");
			ExitCode::FAILURE
		}
	}
}
