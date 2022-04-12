use locspan::Loc;
use static_iref::iri;
use std::collections::HashMap;
use std::path::Path;
use treeldr_vocab::{GraphLabel, Id, StrippedObject, Term, Vocabulary};

fn infallible<T>(t: T) -> Result<T, std::convert::Infallible> {
	Ok(t)
}

#[derive(Default)]
struct BlankIdGenerator(HashMap<rdf_types::BlankIdBuf, treeldr_vocab::BlankLabel>);

impl BlankIdGenerator {
	pub fn generate(&mut self, label: rdf_types::BlankIdBuf) -> treeldr_vocab::BlankLabel {
		use std::collections::hash_map::Entry;
		let len = self.0.len() as u32;
		match self.0.entry(label) {
			Entry::Occupied(entry) => entry.get().clone(),
			Entry::Vacant(entry) => {
				let label = treeldr_vocab::BlankLabel::new(len);
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
	let Loc(quads, _) = Document::parse(&mut lexer).expect("parse error");

	let mut generator = BlankIdGenerator::default();
	let mut generate = move |label| generator.generate(label);

	quads
		.into_iter()
		.map(move |quad| treeldr_vocab::stripped_loc_quad_from_rdf(quad, vocabulary, &mut generate))
		.collect()
}

fn parse_treeldr<P: AsRef<Path>>(
	vocab: &mut Vocabulary,
	path: P,
) -> grdf::HashDataset<Id, Term, StrippedObject, GraphLabel> {
	use treeldr_syntax::{build, Build, Document, Lexer, Parse};

	let input = std::fs::read_to_string(path).expect("unable to read input file");
	let mut lexer = Lexer::new((), input.chars().map(infallible));
	let ast = Document::parse(&mut lexer).expect("parse error");
	let mut context = build::Context::new(vocab, Some(iri!("http://www.example.com").into()));
	let mut quads = Vec::new();
	ast.build(&mut context, &mut quads).expect("build error");

	quads.into_iter().map(treeldr_vocab::strip_quad).collect()
}

fn test<I: AsRef<Path>, O: AsRef<Path>>(input_path: I, expected_output_path: O) {
	use treeldr_vocab::RdfDisplay;
	let mut vocabulary = Vocabulary::new();
	let output = parse_treeldr(&mut vocabulary, input_path);
	let expected_output = parse_nquads(&mut vocabulary, expected_output_path);

	for quad in output.quads() {
		println!("{} .", quad.rdf_display(&vocabulary))
	}

	assert!(output.is_isomorphic_to(&expected_output))
}

#[test]
fn t001() {
	test("tests/001-in.tldr", "tests/001-out.nq")
}

#[test]
fn t002() {
	test("tests/002-in.tldr", "tests/002-out.nq")
}

#[test]
fn t003() {
	test("tests/003-in.tldr", "tests/003-out.nq")
}

#[test]
fn t004() {
	test("tests/004-in.tldr", "tests/004-out.nq")
}

#[test]
fn t005() {
	test("tests/005-in.tldr", "tests/005-out.nq")
}

#[test]
fn t007() {
	test("tests/007-in.tldr", "tests/007-out.nq")
}

#[test]
fn t008() {
	test("tests/008-in.tldr", "tests/008-out.nq")
}
