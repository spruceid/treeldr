use iref::Iri;
use nquads_syntax::Parse;
use rdf_types::{IriInterpretationMut, IriVocabularyMut};
use static_iref::iri;
use std::fs;
use std::path::PathBuf;

fn file_path(id: &str, suffix: &str) -> PathBuf {
	format!("{}/tests/hy/{id}{suffix}", env!("CARGO_MANIFEST_DIR")).into()
}

fn test<const N: usize>(id: &str, inputs: [&Iri; N]) {
	// File paths.
	let input_path = file_path(id, "-in.nq");
	let layout_path = file_path(id, "-layout.json");
	let output_path = file_path(id, "-out.json");

	// RDF vocabulary and interpretation.
	let mut vocabulary = ();
	let mut interpretation =
		rdf_types::interpretation::WithGenerator::new((), rdf_types::generator::Blank::new());

	// Preparet the context to parse the layout.
	let mut builder = treeldr_build::Builder::new();
	let mut context = builder.with_interpretation_mut(&mut vocabulary, &mut interpretation);

	// Parse the input dataset from N-Quads.
	let dataset: grdf::BTreeDataset =
		nquads_syntax::Document::parse_str(&std::fs::read_to_string(input_path).unwrap(), |span| {
			span
		})
		.unwrap()
		.into_value()
		.into_iter()
		.map(|q| q.into_value().strip_all_but_predicate().into_grdf())
		.collect();

	// Parse the layout definition.
	let layout_ref = serde_json::from_str::<treeldr_build::syntax::Layout>(
		&fs::read_to_string(layout_path).unwrap(),
	)
	.unwrap()
	.build(&mut context)
	.unwrap();

	// Parse the expected output.
	let expected: treeldr::Value =
		serde_json::from_str(&fs::read_to_string(output_path).unwrap()).unwrap();

	// Compile the layouts.
	let layouts = builder.build();

	// Parse the inputs.
	let inputs: Vec<_> = inputs
		.into_iter()
		.map(|iri| interpretation.interpret_iri(vocabulary.insert(iri)))
		.collect();

	// Hydrate.
	let output = treeldr_distiller::hydrate(
		&vocabulary,
		&interpretation,
		&layouts,
		&dataset,
		None,
		&layout_ref,
		&inputs,
	)
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
	test("t02", [iri!("https://example.org/#receipt")])
}
