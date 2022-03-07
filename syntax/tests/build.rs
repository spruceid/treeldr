use grdf::Dataset;
use locspan::{Loc, Strip};
use rdf_types::Quad;
use static_iref::iri;
use std::collections::HashMap;
use std::path::Path;

fn infallible<T>(t: T) -> Result<T, std::convert::Infallible> {
	Ok(t)
}

#[derive(Default)]
struct BlankIdGenerator(HashMap<treeldr_syntax::build::BlankLabel, rdf_types::BlankIdBuf>);

impl BlankIdGenerator {
	pub fn generate(&mut self, label: treeldr_syntax::build::BlankLabel) -> rdf_types::BlankIdBuf {
		use std::collections::hash_map::Entry;
		match self.0.entry(label) {
			Entry::Occupied(entry) => entry.get().clone(),
			Entry::Vacant(entry) => {
				let label = rdf_types::BlankIdBuf::from_u32(label.index());
				entry.insert(label.clone());
				label
			}
		}
	}
}

fn parse_nquads<P: AsRef<Path>>(path: P) -> grdf::HashDataset {
	use nquads_syntax::{lexing::Utf8Decoded, Document, Lexer, Parse};

	let buffer = std::fs::read_to_string(path).expect("unable to read file");
	let mut lexer = Lexer::new(
		(),
		Utf8Decoded::new(buffer.chars().map(infallible)).peekable(),
	);
	let Loc(quads, _) = Document::parse(&mut lexer).expect("parse error");

	quads
		.into_iter()
		.map(|Loc(Quad(s, p, o, g), _)| {
			Quad(
				s.into_value().into_term(),
				rdf_types::Term::Iri(p.into_value()),
				o.strip(),
				g.map(|g| g.into_value().into_term()),
			)
		})
		.collect()
}

fn parse_treeldr<P: AsRef<Path>>(path: P) -> grdf::HashDataset {
	use treeldr_syntax::{build, Build, Document, Lexer, Parse};

	let input = std::fs::read_to_string(path).expect("unable to read input file");
	let mut lexer = Lexer::new((), input.chars().map(infallible));
	let ast = Document::parse(&mut lexer).expect("parse error");
	let mut context = build::Context::new(iri!("http://www.example.com").into());
	let mut quads = Vec::new();
	ast.build(&mut context, &mut quads).expect("build error");

	let mut generator = BlankIdGenerator::default();
	let mut generate = move |label| generator.generate(label);

	quads
		.into_iter()
		.map(|Loc(Quad(s, p, o, g), _)| {
			Quad(
				s.into_value().into_grdf(context.namespace(), &mut generate),
				rdf_types::Term::Iri(p.into_value().iri(context.namespace()).unwrap().into()),
				o.strip().into_grdf(context.namespace(), &mut generate),
				g.map(|g| g.into_value().into_grdf(context.namespace(), &mut generate)),
			)
		})
		.collect()
}

fn test<I: AsRef<Path>, O: AsRef<Path>>(input_path: I, expected_output_path: O) {
	let output = parse_treeldr(input_path);
	let expected_output = parse_nquads(expected_output_path);

	for quad in output.quads() {
		println!("{}", quad)
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
