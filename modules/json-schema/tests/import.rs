use iref::Iri;
use locspan::Loc;
use static_iref::iri;
use std::collections::HashMap;
use std::path::Path;
use treeldr::vocab::{BlankLabel, GraphLabel, Id, StrippedObject, Term, Vocabulary};
use treeldr_build::Context;

fn infallible<T>(t: T) -> Result<T, std::convert::Infallible> {
	Ok(t)
}

#[derive(Default)]
struct BlankIdGenerator(HashMap<rdf_types::BlankIdBuf, BlankLabel>);

impl BlankIdGenerator {
	pub fn generate(&mut self, label: rdf_types::BlankIdBuf) -> BlankLabel {
		use std::collections::hash_map::Entry;
		let len = self.0.len() as u32;
		match self.0.entry(label) {
			Entry::Occupied(entry) => entry.get().clone(),
			Entry::Vacant(entry) => {
				let label = BlankLabel::new(len);
				entry.insert(label);
				label
			}
		}
	}
}

fn parse_nquads<P: AsRef<Path>>(
	vocabulary: &mut Vocabulary,
	path: P,
) -> grdf::HashDataset<Id, Term, StrippedObject, GraphLabel> {
	use nquads_syntax::{lexing::Utf8Decoded, Document, Lexer, Parse};

	let buffer = std::fs::read_to_string(path).expect("unable to read file");
	let mut lexer = Lexer::new(
		(),
		Utf8Decoded::new(buffer.chars().map(infallible)).peekable(),
	);
	let quads = Document::parse(&mut lexer)
		.expect("parse error")
		.into_value();

	let mut generator = BlankIdGenerator::default();
	let mut generate = move |label| generator.generate(label);

	quads
		.into_iter()
		.map(move |quad| {
			treeldr::vocab::stripped_loc_quad_from_rdf(quad, vocabulary, &mut generate)
		})
		.collect()
}

fn parse_json_schema<P: AsRef<Path>>(path: P) -> treeldr_json_schema::Schema {
	let buffer = std::fs::read_to_string(path).expect("unable to read file");
	let json: serde_json::Value = serde_json::from_str(&buffer).expect("invalid JSON");
	treeldr_json_schema::Schema::try_from(json).expect("invalid JSON Schema")
}

fn import_json_schema<P: AsRef<Path>>(
	vocabulary: &mut Vocabulary,
	path: P,
) -> (grdf::HashDataset<Id, Term, StrippedObject, GraphLabel>, Id) {
	let input = parse_json_schema(path);
	let mut context: Context<()> = Context::new();
	context.apply_built_in_definitions(vocabulary).unwrap();

	let id = treeldr_json_schema::import_schema(&input, None, &mut context, vocabulary)
		.expect("import failed");
	let model = context.build(vocabulary).expect("build failed");

	let mut quads = Vec::new();
	model.to_rdf(vocabulary, &mut quads);
	(quads.into_iter().collect(), id)
}

fn test<I: AsRef<Path>, O: AsRef<Path>>(input_path: I, expected_output_path: O, expected_iri: Iri) {
	use treeldr::vocab::RdfDisplay;
	let mut vocabulary = Vocabulary::new();
	let expected_id = Id::Iri(Term::from_iri(expected_iri.into(), &mut vocabulary));

	let (output, id) = import_json_schema(&mut vocabulary, input_path);
	let expected_output = parse_nquads(&mut vocabulary, expected_output_path);

	for quad in output.quads() {
		eprintln!("{} .", quad.rdf_display(&vocabulary))
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
