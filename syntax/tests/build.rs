use locspan::Loc;
use static_iref::iri;
use std::collections::HashMap;
use std::path::Path;
use treeldr::vocab::{BlankLabel, GraphLabel, Id, StrippedObject, Term, Vocabulary};

type BuildContext = treeldr_build::Context<(), treeldr_syntax::build::Descriptions>;

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
) -> grdf::BTreeDataset<Id, Term, StrippedObject, GraphLabel> {
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
			treeldr::vocab::stripped_loc_quad_from_rdf(quad, vocabulary, &mut generate)
		})
		.collect()
}

fn parse_treeldr<P: AsRef<Path>>(
	vocabulary: &mut Vocabulary,
	path: P,
) -> grdf::BTreeDataset<Id, Term, StrippedObject, GraphLabel> {
	use treeldr_build::Document;
	use treeldr_syntax::{Lexer, Parse};

	let input = std::fs::read_to_string(path).expect("unable to read input file");
	let mut lexer = Lexer::new((), input.chars().map(infallible));
	let ast = treeldr_syntax::Document::parse(&mut lexer).expect("parse error");
	let mut context = BuildContext::new();
	context.define_rdf_types().unwrap();
	context.define_xml_types().unwrap();
	let mut local_context =
		treeldr_syntax::build::LocalContext::new(Some(iri!("http://www.example.com").into()));
	ast.declare(&mut local_context, &mut context, vocabulary)
		.expect("build error");
	ast.into_value()
		.relate(&mut local_context, &mut context, vocabulary)
		.expect("build error");

	let context = context.simplify(vocabulary).expect("simplification failed");

	let model = context.build(vocabulary).expect("build error");

	let mut quads = Vec::new();
	model.to_rdf(vocabulary, &mut quads);

	quads.into_iter().collect()
}

fn test<I: AsRef<Path>, O: AsRef<Path>>(input_path: I, expected_output_path: O) {
	use treeldr::vocab::RdfDisplay;
	let mut vocabulary = Vocabulary::new();
	let output = parse_treeldr(&mut vocabulary, input_path);
	let expected_output = parse_nquads(&mut vocabulary, expected_output_path);

	for quad in output.quads() {
		println!("{} .", quad.rdf_display(&vocabulary))
	}

	assert!(output.is_isomorphic_to(&expected_output))
}

fn negative_test<I: AsRef<Path>>(input_path: I) {
	use treeldr::vocab::RdfDisplay;
	let mut vocabulary = Vocabulary::new();
	let output = parse_treeldr(&mut vocabulary, input_path);
	for quad in output.quads() {
		println!("{} .", quad.rdf_display(&vocabulary))
	}
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

#[test]
fn t009() {
	test("tests/009-in.tldr", "tests/009-out.nq")
}

#[test]
fn t010() {
	test("tests/010-in.tldr", "tests/010-out.nq")
}

#[test]
fn t011() {
	test("tests/011-in.tldr", "tests/011-out.nq")
}

#[test]
fn t012() {
	test("tests/012-in.tldr", "tests/012-out.nq")
}

// #[test]
// fn t013() {
// 	test("tests/013-in.tldr", "tests/013-out.nq")
// }

#[test]
#[should_panic]
fn e01() {
	negative_test("tests/e01.tldr")
}

#[test]
#[should_panic]
fn e02() {
	negative_test("tests/e02.tldr")
}
