use iref::Iri;
use nquads_syntax::Parse;
use rdf_types::{Id, Term};
use static_iref::iri;
use std::fs;
use std::path::PathBuf;
use treeldr_layouts::utils::strip_rdf_quad;

fn file_path(id: &str, suffix: &str) -> PathBuf {
	format!("{}/tests/hy/{id}{suffix}", env!("CARGO_MANIFEST_DIR")).into()
}

fn test<const N: usize>(id: &str, inputs: [&Iri; N]) {
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

	// Parse the inputs.
	let inputs: Vec<_> = inputs
		.into_iter()
		.map(|iri| Term::Id(Id::Iri(iri.to_owned())))
		.collect();

	// Hydrate.
	let output = treeldr_layouts::distill::hydrate(&layouts, &dataset, &layout_ref, &inputs)
		.unwrap()
		.into_untyped();

	// Test.
	assert_eq!(output, expected)
}

#[test]
fn t01() {
	test("t01", [iri!("https://example.org/#john.smith")])
}

#[test]
fn t02() {
	test("t02", [iri!("https://example.org/#john.smith")])
}

#[test]
fn t03() {
	test("t03", [iri!("https://example.org/#receipt")])
}

#[test]
fn t04() {
	test("t04", [iri!("https://example.org/#receipt")])
}
