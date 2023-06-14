use contextual::WithContext;
use locspan::{Meta, Span};
use nquads_syntax::BlankIdBuf;
use rdf_types::BlankIdVocabularyMut;
use static_iref::iri;
use std::path::Path;
use treeldr::{
	to_rdf::ToRdf,
	vocab::{GraphLabel, Id, StrippedObject, TldrVocabulary},
	BlankIdIndex, IriIndex,
};

type BuildContext = treeldr_build::Context<Span>;

fn parse_nquads<P: AsRef<Path>>(
	vocabulary: &mut TldrVocabulary,
	path: P,
) -> grdf::BTreeDataset<Id, IriIndex, StrippedObject, GraphLabel> {
	use nquads_syntax::{Document, Parse};

	let buffer = std::fs::read_to_string(path).expect("unable to read file");
	let Meta(quads, _) = Document::parse_str(&buffer, |span| span).expect("parse error");

	let generate = move |vocabulary: &mut TldrVocabulary, label: BlankIdBuf| {
		vocabulary.insert_owned_blank_id(label)
	};

	quads
		.into_iter()
		.map(move |quad| treeldr::vocab::stripped_loc_quad_from_rdf(quad, vocabulary, generate))
		.collect()
}

fn parse_treeldr<P: AsRef<Path>>(
	vocabulary: &mut TldrVocabulary,
	path: P,
) -> grdf::BTreeDataset<Id, IriIndex, StrippedObject, GraphLabel> {
	use treeldr_build::Document;
	use treeldr_syntax::Parse;

	let input = std::fs::read_to_string(path).expect("unable to read input file");
	let ast = treeldr_syntax::Document::parse_str(&input, |span| span).expect("parse error");
	let mut context = BuildContext::new();
	let mut generator = rdf_types::generator::Blank::new_with_prefix("t".to_string());
	context.apply_built_in_definitions(vocabulary, &mut generator);
	let mut local_context =
		treeldr_syntax::build::LocalContext::new(Some(iri!("http://www.example.com").into()));

	ast.declare(&mut local_context, &mut context, vocabulary, &mut generator)
		.expect("build error");
	ast.into_value()
		.define(&mut local_context, &mut context, vocabulary, &mut generator)
		.expect("build error");

	let model = context
		.build(vocabulary, &mut generator)
		.expect("build error");

	let mut quads = Vec::new();
	model.to_rdf(vocabulary, &mut generator, &mut quads);

	quads.into_iter().collect()
}

pub enum Test {
	Positive {
		input: &'static str,
		expected_output: &'static str,
	},
	Negative {
		input: &'static str,
	},
}

impl Test {
	fn run(self) {
		match self {
			Self::Positive {
				input,
				expected_output,
			} => {
				let mut vocabulary = rdf_types::IndexVocabulary::<IriIndex, BlankIdIndex>::new();
				let output = parse_treeldr(&mut vocabulary, input);
				let expected_output = parse_nquads(&mut vocabulary, expected_output);

				for quad in output.quads() {
					println!("{} .", quad.with(&vocabulary))
				}

				assert!(output.is_isomorphic_to(&expected_output))
			}
			Self::Negative { input } => {
				let mut vocabulary = rdf_types::IndexVocabulary::<IriIndex, BlankIdIndex>::new();
				let output = parse_treeldr(&mut vocabulary, input);
				for quad in output.quads() {
					println!("{} .", quad.with(&vocabulary))
				}
			}
		}
	}
}

macro_rules! positive {
	{ $($id:ident),* } => {
		$(
			#[test_log::test]
			fn $id () {
				Test::Positive {
					input: concat!("tests/build/", stringify!($id), "-in.tldr"),
					expected_output: concat!("tests/build/", stringify!($id), "-out.nq")
				}.run()
			}
		)*
	};
}

macro_rules! negative {
	{ $($id:ident),* } => {
		$(
			#[test_log::test]
			#[should_panic]
			fn $id () {
				Test::Negative {
					input: concat!("tests/build/", stringify!($id), "-in.tldr")
				}.run()
			}
		)*
	};
}

positive! {
	p001,
	p002,
	p003,
	p004,
	p005,
	p007,
	p008,
	p009,
	p010,
	p011,
	p012,
	p013,
	p014,
	p015,
	p016,
	p017,
	p018
}

negative! {
	n001,
	n002
}
