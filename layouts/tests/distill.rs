use nquads_syntax::Parse;
use paste::paste;
use rdf_types::{BlankIdBuf, Id, Term};
use std::fs;
use std::path::PathBuf;
use treeldr_layouts::utils::strip_rdf_quad;

fn file_path(id: &str, suffix: &str) -> PathBuf {
	format!("{}/tests/distill/{id}{suffix}", env!("CARGO_MANIFEST_DIR")).into()
}

fn hydrate<const N: usize>(id: &str, inputs: [Term; N]) {
	// File paths.
	let input_path = file_path(id, "-in.nq");
	let layout_path = file_path(id, "-layout.json");
	let output_path = file_path(id, "-out.json");

	// Parse the input dataset from N-Quads.
	let dataset: grdf::BTreeDataset =
		nquads_syntax::Document::parse_str(&std::fs::read_to_string(input_path).unwrap(), |span| {
			span
		})
		.unwrap()
		.into_value()
		.into_iter()
		.map(strip_rdf_quad)
		.collect();

	// Initialize the layout builder.
	let mut builder = treeldr_layouts::abs::Builder::new();

	// Parse the layout definition.
	let layout_ref = serde_json::from_str::<treeldr_layouts::abs::syntax::Layout>(
		&fs::read_to_string(layout_path).unwrap(),
	)
	.unwrap()
	.build(&mut builder)
	.unwrap();

	// Parse the expected output.
	let expected: treeldr_layouts::Value =
		serde_json::from_str(&fs::read_to_string(output_path).unwrap()).unwrap();

	// Compile the layouts.
	let layouts = builder.build();

	// Hydrate.
	let output = treeldr_layouts::distill::hydrate(&layouts, &dataset, &layout_ref, &inputs)
		.unwrap()
		.into_untyped();

	// Test.
	assert_eq!(output, expected)
}

fn dehydrate<const N: usize>(id: &str, expected_values: [Term; N]) {
	// File paths.
	let input_path = file_path(id, "-out.json");
	let layout_path = file_path(id, "-layout.json");
	let output_path = file_path(id, "-in.nq");

	// Parse the JSON input.
	let input: treeldr_layouts::Value =
		serde_json::from_str(&fs::read_to_string(input_path).unwrap()).unwrap();

	// Parse the expected output dataset from N-Quads.
	let expected_dataset: grdf::BTreeDataset = nquads_syntax::Document::parse_str(
		&std::fs::read_to_string(output_path).unwrap(),
		|span| span,
	)
	.unwrap()
	.into_value()
	.into_iter()
	.map(strip_rdf_quad)
	.collect();

	// Initialize the layout builder.
	let mut builder = treeldr_layouts::abs::Builder::new();

	// Parse the layout definition.
	let layout_ref = serde_json::from_str::<treeldr_layouts::abs::syntax::Layout>(
		&fs::read_to_string(layout_path).unwrap(),
	)
	.unwrap()
	.build(&mut builder)
	.unwrap();

	// Compile the layouts.
	let layouts = builder.build();

	// Hydrate.
	let (output_dataset, output_values) = treeldr_layouts::distill::dehydrate(
		&layouts,
		&input,
		&layout_ref,
		Some(expected_values.len() as u32),
	)
	.unwrap();

	for quad in &output_dataset {
		println!("{quad} .")
	}

	// Test.
	let bijection = output_dataset
		.find_blank_id_bijection(&expected_dataset)
		.unwrap(); // fail if the output is not isomorphic to the expected dataset.

	assert_eq!(output_values.len(), expected_values.len());
	for (output, expected) in output_values.iter().zip(&expected_values) {
		match (output, expected) {
			(Term::Id(Id::Blank(output)), Term::Id(Id::Blank(expected))) => {
				assert_eq!(*bijection.forward.get(output).unwrap(), expected)
			}
			(output, expected) => {
				assert_eq!(output, expected)
			}
		}
	}
}

macro_rules! test {
	($name:ident ($($e:expr),*)) => {
		paste! {
			#[test]
			fn [<hydrate_ $name>] () {
				hydrate(stringify!($name), [$($e),*])
			}
		}

		paste! {
			#[test]
			fn [<dehydrate_ $name>] () {
				dehydrate(stringify!($name), [$($e),*])
			}
		}
	};
}

test! {
	t01 (Term::blank(BlankIdBuf::new("_:john_smith".to_string()).unwrap()))
}

test! {
	t02 (Term::blank(BlankIdBuf::new("_:john_smith".to_string()).unwrap()))
}

test! {
	t03 (Term::blank(BlankIdBuf::new("_:receipt".to_string()).unwrap()))
}

test! {
	t04 (Term::blank(BlankIdBuf::new("_:receipt".to_string()).unwrap()))
}
