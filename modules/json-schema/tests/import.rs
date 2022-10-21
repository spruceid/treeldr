use contextual::WithContext;
use iref::Iri;
use locspan::Meta;
use rdf_types::{BlankIdBuf, IriVocabularyMut, VocabularyMut};
use static_iref::iri;
use std::path::Path;
use treeldr::{
	vocab::{GraphLabel, Id, StrippedObject},
	BlankIdIndex, IriIndex,
};
use treeldr_build::Context;

fn infallible<T>(t: T) -> Result<T, std::convert::Infallible> {
	Ok(t)
}

fn parse_nquads<P: AsRef<Path>, V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
	vocabulary: &mut V,
	path: P,
) -> grdf::BTreeDataset<Id, IriIndex, StrippedObject, GraphLabel> {
	use nquads_syntax::{lexing::Utf8Decoded, Document, Lexer, Parse};

	let buffer = std::fs::read_to_string(path).expect("unable to read file");
	let mut lexer = Lexer::new(
		(),
		Utf8Decoded::new(buffer.chars().map(infallible)).peekable(),
	);
	let Meta(quads, _) = Document::parse(&mut lexer).expect("parse error");

	let generate = move |vocabulary: &mut V, label: BlankIdBuf| {
		vocabulary.insert_blank_id(label.as_blank_id_ref())
	};

	quads
		.into_iter()
		.map(move |quad| treeldr::vocab::stripped_loc_quad_from_rdf(quad, vocabulary, generate))
		.collect()
}

fn parse_json_schema<P: AsRef<Path>>(path: P) -> treeldr_json_schema::Schema {
	let buffer = std::fs::read_to_string(path).expect("unable to read file");
	let json: serde_json::Value = serde_json::from_str(&buffer).expect("invalid JSON");
	treeldr_json_schema::Schema::try_from(json).expect("invalid JSON Schema")
}

fn import_json_schema<P: AsRef<Path>, V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
	vocabulary: &mut V,
	path: P,
) -> (
	grdf::BTreeDataset<Id, IriIndex, StrippedObject, GraphLabel>,
	Id,
) {
	let input = parse_json_schema(path);
	let mut context: Context<()> = Context::new();
	let mut generator = rdf_types::generator::Blank::new_with_prefix("t".to_string());
	context
		.apply_built_in_definitions(vocabulary, &mut generator)
		.unwrap();
	let id =
		treeldr_json_schema::import_schema(&input, None, &mut context, vocabulary, &mut generator)
			.expect("import failed");
	let model = context
		.build(vocabulary, &mut generator)
		.expect("build failed");

	let mut quads = Vec::new();
	model.to_rdf(vocabulary, &mut generator, &mut quads);
	(quads.into_iter().collect(), id)
}

fn test<I: AsRef<Path>, O: AsRef<Path>>(input_path: I, expected_output_path: O, expected_iri: Iri) {
	let mut vocabulary = rdf_types::IndexVocabulary::<IriIndex, BlankIdIndex>::new();
	let expected_id = Id::Iri(vocabulary.insert(expected_iri));

	let (output, id) = import_json_schema(&mut vocabulary, input_path);
	let expected_output = parse_nquads(&mut vocabulary, expected_output_path);

	for quad in output.quads() {
		eprintln!("{} .", quad.with(&vocabulary))
	}

	assert!(output.is_isomorphic_to(&expected_output));
	assert_eq!(id, expected_id)
}

#[test]
fn t001() {
	test(
		"tests/i01.json",
		"tests/i01.nq",
		iri!("https://treeldr.org/String"),
	)
}

#[test]
fn t002() {
	test(
		"tests/i02.json",
		"tests/i02.nq",
		iri!("https://example.com/product.schema.json"),
	)
}

#[test]
fn t003() {
	test(
		"tests/i03.json",
		"tests/i03.nq",
		iri!("https://example.com/product.schema.json"),
	)
}

#[test]
fn t004() {
	test(
		"tests/i04.json",
		"tests/i04.nq",
		iri!("https://example.com/product.schema.json"),
	)
}

#[test]
fn t005() {
	test(
		"tests/i05.json",
		"tests/i05.nq",
		iri!("https://example.com/product.schema.json"),
	)
}

#[test]
fn t006() {
	test(
		"tests/i06.json",
		"tests/i06.nq",
		iri!("https://example.com/foo.schema.json"),
	)
}
