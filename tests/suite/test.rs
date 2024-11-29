use std::{
	fs,
	path::{Path, PathBuf},
};

use nquads_syntax::Parse;
use rdf_types::{
	dataset::{BTreeDataset, IndexedBTreeDataset},
	Id, Term,
};
use treeldr::{
	rdf::{Rdf, RdfContextMut},
	utils::strip_rdf_quad,
	Package, Registry, RegistryUnion, Value,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum PackageId {
	Prelude,
	Target,
}

impl PositiveTest {
	pub fn run(&self, id: &str, forward: bool) {
		let mut rdf = Rdf::default();
		let prelude = Package::prelude(PackageId::Prelude);

		// Parse layout.
		let layout_text = fs::read_to_string(file_path(format!("{id}-layout.tldr")))
			.expect("unable to read layout");
		let package = treeldr::Module::compile(&mut rdf, &prelude, PackageId::Target, &layout_text)
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
		let input: IndexedBTreeDataset = nquads_syntax::Document::parse_str(&input_text)
			.unwrap()
			.into_value()
			.into_iter()
			.map(strip_rdf_quad)
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
				.call(&registry, &rdf, &input, &args)
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
			let mut found = BTreeDataset::new();
			let found_args = function
				.call_inverse(&registry, &mut rdf, &mut found, &[], &output)
				.expect("inverse evaluation failed");

			for (i, a) in found_args.iter().enumerate() {
				if let Value::Resource(a @ Term::Id(Id::Blank(_))) = a {
					if let Value::Resource(b @ Term::Id(Id::Iri(_))) = &args[i] {
						rdf.unify(a, b).expect("term unification failed");
					}
				}
			}

			let found: BTreeDataset = found
				.into_iter()
				.map(|quad| quad.map(|t| rdf.interpretation.map_term(t)))
				.collect();

			let found_args: Vec<_> = found_args
				.into_iter()
				.map(|value| match value {
					Value::Resource(r) => Value::Resource(rdf.interpretation.map_term(r)),
					other => other,
				})
				.collect();

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
					Value::Resource(term @ Term::Id(Id::Blank(_))) => {
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
