use clap::Parser;
use codespan_reporting::term::{termcolor::StandardStream, self};
use iref::IriBuf;
use quote::format_ident;
use rdf_types::IriVocabulary;
use stderrlog::ColorChoice;
use treeldr::{TId, Id};
use treeldr_rust_gen::Generate;
use std::{path::PathBuf, collections::HashMap};
use treeldr_load as load;

#[derive(Parser)]
#[clap(name="treeldr", author, version, about, long_about = None)]
struct Args {
	/// Input files.
	#[clap(short = 'i', multiple_occurrences = true)]
	filenames: Vec<PathBuf>,

	/// Sets the level of verbosity.
	#[clap(short, long = "verbose", parse(from_occurrences))]
	verbosity: usize,

	/// Layouts to generate.
	layouts: Vec<IriBuf>,
}

pub fn main() {
	// Parse options.
	let args = Args::parse();

	// Init logger.
	stderrlog::new().verbosity(args.verbosity).init().unwrap();

	let mut files = load::Files::<PathBuf>::new();
	let mut documents = Vec::new();

	for filename in args.filenames {
		match load::Document::load(&mut files, &filename) {
			Ok((doc, _)) => {
				documents.push(doc)
			}
			Err(e) => {
				eprintln!("error: {e}")
			}
		}
	}

	let mut vocabulary = rdf_types::IndexVocabulary::new();
	let mut generator = rdf_types::generator::Blank::new();
	let mut build_context = load::BuildContext::new();

	match load::build_all(
		&mut vocabulary,
		&mut generator,
		&mut build_context,
		documents,
	) {
		Ok(model) => {
			let mut gen_context =
				treeldr_rust_gen::Context::new(&model, &vocabulary);

			let module_ref = gen_context.add_module(None, format_ident!("example"));

			let mut map = HashMap::new();
			for layout_iri in args.layouts {
				let layout_ref = TId::new(Id::Iri(vocabulary.get(layout_iri.as_iri()).unwrap()));
				map.insert(layout_ref, treeldr_rust_gen::module::Parent::Ref(module_ref));
			}

			for (layout_ref, _) in model.layouts() {
				gen_context.add_layout(
					map.get(&layout_ref)
						.cloned()
						.or(Some(treeldr_rust_gen::module::Parent::Extern)),
					layout_ref,
				)
			}

			let module = gen_context.module(module_ref).unwrap();
			let generated = module.generate_with(&gen_context, Some(module_ref)).into_tokens().unwrap();
			println!("{generated}")
		}
		Err(e) => {
			use load::reporting::Diagnose;

			let diagnostic = contextual::WithContext::with(&e, &vocabulary).diagnostic();
			let writer = StandardStream::stderr(ColorChoice::Always);
			let config = codespan_reporting::term::Config::default();
			term::emit(&mut writer.lock(), &config, &files, &diagnostic)
				.expect("diagnostic failed");
		}
	}
}
