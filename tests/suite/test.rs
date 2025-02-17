use std::{
	fs,
	path::{Path, PathBuf},
};

use nquads_syntax::Parse;
use rdf_types::{
	dataset::IndexedBTreeDataset,
	generator::{self, LocalGeneratorInterpretation},
	LocalTerm, Term,
};
use treeldr::{
	rdf::{FallibleResourceUnifier, SubstituteResource, TryUnifyInterpretation},
	Package, PreludeOrTarget, Registry, RegistryUnion, Value,
};

fn file_path(path: impl AsRef<Path>) -> PathBuf {
	let mut absolute_path: PathBuf = env!("CARGO_MANIFEST_DIR").into();
	absolute_path.push("tests");
	absolute_path.push("suite");
	absolute_path.extend(path.as_ref());
	absolute_path
}

pub struct PositiveTest {
	pub inputs: Vec<&'static str>,
}

impl PositiveTest {
	pub fn run(&self, id: &str, forward: bool) {
		// let mut rdf = Rdf::default();
		let mut generator = generator::Blank::new();
		let prelude = Package::prelude(PreludeOrTarget::Prelude);

		// Parse layout.
		let layout_text = fs::read_to_string(file_path(format!("{id}-layout.tldr")))
			.expect("unable to read layout");
		let package = treeldr::Package::compile(&prelude, PreludeOrTarget::Target, &layout_text)
			.expect("compile error");

		let registry = RegistryUnion(prelude, &package);

		let f_name = "_";
		let f_ref = package
			.root()
			.get_by_name(f_name)
			.unwrap()
			.as_function()
			.unwrap();
		let function = registry.get_function(f_ref).unwrap();

		// Parse input.
		let input_text = fs::read_to_string(file_path(format!("{id}-in.nq")))
			.expect("unable to read RDF dataset");
		let mut input: IndexedBTreeDataset = nquads_syntax::GrdfDocument::parse_str(&input_text)
			.unwrap()
			.0
			.into_iter()
			.collect();

		// Parse output.
		let output_text: serde_json::Value =
			fs::read_to_string(file_path(format!("{id}-out.json")))
				.expect("unable to read TreeLDR value")
				.parse()
				.expect("unable to parse JSON");
		let output: treeldr::Value = output_text.into();

		// Parse input arguments.
		let args: Vec<_> = self
			.inputs
			.iter()
			.map(|a| a.parse().expect("unable to parse input argument"))
			.collect();

		if forward {
			// Evaluate.
			let found = function
				.call(&registry, &mut generator, &mut input, &args)
				.expect("evaluation failed");

			if output != found {
				eprintln!("expected: {output}");
				eprintln!("found   : {found}");
				panic!("found value is different from the expected value");
			}

			let _ = found
				.into_typed(&registry, function.signature.return_type)
				.expect("post typing failed");
		} else {
			let mut found = IndexedBTreeDataset::new();
			let mut found_args = function
				.call_inverse(&registry, &mut generator, &mut found, &[], &output)
				.expect("inverse evaluation failed");

			let mut interpretation = LocalGeneratorInterpretation::new(&mut generator);
			let mut unifier = interpretation.begin_unification();
			for (i, a) in found_args.iter().enumerate() {
				if let Value::Resource(a @ LocalTerm::Anonymous(_)) = a {
					if let Value::Resource(b @ LocalTerm::Named(Term::Iri(_))) = &args[i] {
						unifier.try_unify(a, b).expect("term unification failed");
					}
				}
			}

			found.substitute_resource(|r| unifier.substitute_resource(r));
			found_args.substitute_resource(|r| unifier.substitute_resource(r));

			let Some(bijection) = rdf_types::dataset::isomorphism::find_bijection(&found, &input)
			else {
				eprintln!("expected dataset:");
				for quad in input {
					eprintln!("{quad} .");
				}

				eprintln!("\nexpected arguments:");
				for arg in args {
					eprintln!("{arg}");
				}

				eprintln!("\nfound dataset:");
				for quad in found {
					eprintln!("{quad} .");
				}

				eprintln!("\nfound arguments:");
				for arg in found_args {
					eprintln!("{arg}");
				}

				panic!("output dataset is not isomorphic to the expected dataset");
			};

			let found_args: Vec<_> = found_args
				.into_iter()
				.map(|value| match value {
					Value::Resource(term @ LocalTerm::Anonymous(_)) => {
						Value::Resource((*bijection.forward.get(&term).unwrap()).clone())
					}
					other => other,
				})
				.collect();

			if found_args != args {
				eprintln!("expected:");
				for arg in args {
					eprintln!("{arg}");
				}

				eprintln!("\nfound:");
				for arg in found_args {
					eprintln!("{arg}");
				}

				panic!("output arguments are different from the expected arguments");
			}
		}
	}
}

macro_rules! positive_tests {
	{ $id:ident, $($input:literal),* } => {
		mod $id {
			use super::*;

			#[test]
			fn forward() {
				let test = PositiveTest {
					inputs: vec![$($input),*]
				};

				test.run(stringify!($id), true);
			}

			#[test]
			fn backward() {
				let test = PositiveTest {
					inputs: vec![$($input),*]
				};

				test.run(stringify!($id), false);
			}
		}
	};
}

pub(crate) use positive_tests;
