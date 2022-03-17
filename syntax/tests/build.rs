use locspan::Loc;
use static_iref::iri;
use std::collections::HashMap;
use std::path::Path;
use treeldr_vocab::{GraphLabel, Name, Vocabulary, StrippedObject, Id};

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
	path: P,
	vocabulary: &mut Vocabulary,
) -> grdf::HashDataset<Id, Name, StrippedObject, GraphLabel> {
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
		.map(move |quad| {
			treeldr_vocab::stripped_loc_quad_from_rdf(quad, vocabulary, &mut generate)
		})
		.collect()
}

fn parse_treeldr<P: AsRef<Path>>(
	path: P,
) -> (
	grdf::HashDataset<Id, Name, StrippedObject, GraphLabel>,
	Vocabulary,
) {
	use treeldr_syntax::{build, Build, Document, Lexer, Parse};

	let input = std::fs::read_to_string(path).expect("unable to read input file");
	let mut lexer = Lexer::new((), input.chars().map(infallible));
	let ast = Document::parse(&mut lexer).expect("parse error");
	let mut context = build::Context::new(Some(iri!("http://www.example.com").into()));
	let mut quads = Vec::new();
	ast.build(&mut context, &mut quads).expect("build error");

	(
		quads
			.into_iter()
			.map(treeldr_vocab::strip_quad)
			.collect(),
		context.into_vocabulary(),
	)
}

fn test<I: AsRef<Path>, O: AsRef<Path>>(input_path: I, expected_output_path: O) {
	use treeldr_vocab::Display;
	let (output, mut vocabulary) = parse_treeldr(input_path);
	let expected_output = parse_nquads(expected_output_path, &mut vocabulary);

	for quad in output.quads() {
		println!("{} .", quad.display(&vocabulary))
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
