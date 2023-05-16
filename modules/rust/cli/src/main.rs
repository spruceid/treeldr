use clap::Parser;
use codespan_reporting::term::{self, termcolor::StandardStream};
use contextual::WithContext;
use iref::IriBuf;
use proc_macro2::{Ident, Span};
use quote::{format_ident, ToTokens};
use rdf_types::IriVocabulary;
use std::{collections::HashMap, fmt, path::PathBuf, str::FromStr};
use stderrlog::ColorChoice;
use treeldr::{Id, TId};
use treeldr_load as load;
use treeldr_rust_gen::{module::Visibility, tr::TraitModules, DedicatedSubModule, GenerateSyntax};

#[derive(Parser)]
#[clap(name="treeldr", author, version, about, long_about = None)]
struct Args {
	/// Input files.
	#[clap(short = 'i')]
	filenames: Vec<PathBuf>,

	/// Sets the level of verbosity.
	#[clap(short, long = "verbose", action = clap::ArgAction::Count)]
	verbosity: u8,

	/// Layouts to generate.
	layouts: Vec<IriBuf>,

	#[clap(short = 'm')]
	modules: Vec<ModuleBinding>,

	#[clap(long)]
	no_rdf: bool,
}

#[derive(Debug, Clone)]
pub struct ModuleBinding {
	pub ident: String,
	pub iri: IriBuf,
}

impl FromStr for ModuleBinding {
	type Err = InvalidPrefixBinding;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.split_once('=') {
			Some((prefix, iri)) => {
				let iri = IriBuf::new(iri)
					.map_err(|e| InvalidPrefixBinding::InvalidIri(iri.to_string(), e))?;
				Ok(Self {
					ident: prefix.to_string(),
					iri,
				})
			}
			None => Err(InvalidPrefixBinding::MissingSeparator(s.to_string())),
		}
	}
}

#[derive(Debug)]
pub enum InvalidPrefixBinding {
	MissingSeparator(String),
	InvalidIri(String, iref::Error),
}

impl std::error::Error for InvalidPrefixBinding {}

impl fmt::Display for InvalidPrefixBinding {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::MissingSeparator(s) => write!(f, "missing separator `=` in `{s}`"),
			Self::InvalidIri(i, e) => write!(f, "invalid IRI `{i}`: {e}"),
		}
	}
}

pub fn main() {
	// Parse options.
	let args = Args::parse();

	// Init logger.
	stderrlog::new()
		.verbosity(args.verbosity as usize)
		.init()
		.unwrap();

	let mut files = load::Files::<PathBuf>::new();
	let mut documents = Vec::new();

	for filename in args.filenames {
		match load::Document::load(&mut files, &filename) {
			Ok((doc, _)) => documents.push(doc),
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
			// let mut quads = Vec::new();
			// model.to_rdf_with(
			// 	&mut vocabulary,
			// 	&mut generator,
			// 	&mut quads,
			// 	treeldr::to_rdf::Options {
			// 		ignore_standard_vocabulary: true
			// 	}
			// );

			// quads.sort();

			// for quad in quads {
			// 	println!("{} .", quad.with(&vocabulary))
			// }

			let options = treeldr_rust_gen::Options {
				impl_rdf: !args.no_rdf,
			};

			let mut gen_context = treeldr_rust_gen::Context::new(&model, &vocabulary, options);

			let root_ref =
				gen_context.add_module(None, None, format_ident!("example"), Visibility::Public);

			let mut layout_map = HashMap::new();
			let mut type_map = HashMap::new();

			for prefix in args.modules {
				let module_ref = gen_context.add_module(
					Some(root_ref),
					None,
					Ident::new(&prefix.ident, Span::call_site()),
					Visibility::Public,
				);

				let mut sub_modules = treeldr_rust_gen::ModulePathBuilder::new(module_ref);

				for (id, node) in model.nodes() {
					if let treeldr::Id::Iri(term) = id {
						let iri = vocabulary.iri(&term).unwrap();

						if let Some(suffix) = iri.as_str().strip_prefix(prefix.iri.as_str()) {
							let path =
								treeldr_rust_gen::ModulePathBuilder::split_iri_path(suffix).0;
							eprintln!("path: {path}");

							if node.is_type() {
								type_map.insert(
									TId::new(id),
									TraitModules {
										main: Some(treeldr_rust_gen::module::Parent::Ref(
											sub_modules.get(&mut gen_context, path, None),
										)),
										provider: Some(treeldr_rust_gen::module::Parent::Ref(
											sub_modules.get(
												&mut gen_context,
												path,
												Some(DedicatedSubModule::ClassProviders),
											),
										)),
										trait_object: Some(treeldr_rust_gen::module::Parent::Ref(
											sub_modules.get(
												&mut gen_context,
												path,
												Some(DedicatedSubModule::TraitObjects),
											),
										)),
									},
								);
							}

							if node.is_layout() {
								layout_map.insert(
									TId::new(id),
									treeldr_rust_gen::module::Parent::Ref(sub_modules.get(
										&mut gen_context,
										path,
										options.impl_rdf.then_some(DedicatedSubModule::Layouts),
									)),
								);
							}
						}
					}
				}
			}

			for layout_iri in args.layouts {
				let layout_ref = TId::new(Id::Iri(vocabulary.get(layout_iri.as_iri()).unwrap()));
				layout_map.insert(layout_ref, treeldr_rust_gen::module::Parent::Ref(root_ref));
			}

			for (id, node) in model.nodes() {
				if node.is_type() {
					let type_ref = TId::new(id);
					gen_context.add_type(
						type_map.get(&type_ref).cloned().unwrap_or_default(),
						type_ref,
					);
				}

				if node.is_layout() {
					let layout_ref = TId::new(id);
					gen_context.add_layout(
						layout_map
							.get(&layout_ref)
							.cloned()
							.or(Some(treeldr_rust_gen::module::Parent::Extern)),
						layout_ref,
					)
				}
			}

			gen_context.run_pre_computations();
			let module = gen_context.module(root_ref).unwrap();
			let scope = treeldr_rust_gen::Scope::new(Some(root_ref));
			match module
				.generate_syntax(&gen_context, &scope)
			{
				Ok(generated) => {
					println!("{}", generated.into_token_stream())
				}
				Err(e) => {
					if let treeldr_rust_gen::Error::UnreachableType(layout_ref) = &e {
						let ty = gen_context.layout_type(*layout_ref);
						eprintln!("unreachable {ty:?}")
					}

					eprintln!("error: {}", e.with(&vocabulary))
				}
			}
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
