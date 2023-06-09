use contextual::WithContext;
use locspan::{Meta, Span};
use nquads_syntax::BlankIdBuf;
use rdf_types::VocabularyMut;
use std::path::Path;
use treeldr::{
	to_rdf::ToRdf,
	vocab::{GraphLabel, Id, Object, StrippedObject, IndexedVocabulary},
	BlankIdIndex, IriIndex,
};
use treeldr_build::Document;

type BuildContext = treeldr_build::Context<Span>;

fn parse_nquads<P: AsRef<Path>, V: IndexedVocabulary + VocabularyMut>(
	vocabulary: &mut V,
	path: P,
) -> grdf::BTreeDataset<Id, IriIndex, StrippedObject, GraphLabel> {
	use nquads_syntax::{Document, Parse};

	let buffer = std::fs::read_to_string(path).expect("unable to read file");
	let Meta(quads, _) = Document::parse_str(&buffer, |span| span).expect("parse error");

	let generate = move |vocabulary: &mut V, label: BlankIdBuf| {
		vocabulary.insert_blank_id(label.as_blank_id_ref())
	};

	quads
		.into_iter()
		.map(move |quad| treeldr::vocab::stripped_loc_quad_from_rdf(quad, vocabulary, generate))
		.collect()
}

fn parse_meta_nquads<P: AsRef<Path>, V: IndexedVocabulary + VocabularyMut>(
	vocabulary: &mut V,
	path: P,
) -> grdf::meta::BTreeDataset<Id, IriIndex, Object<Span>, GraphLabel, Span> {
	use nquads_syntax::{Document, Parse};

	let buffer = std::fs::read_to_string(path).expect("unable to read file");
	let Meta(quads, _) = Document::parse_str(&buffer, |span| span).expect("parse error");

	let generate = move |vocabulary: &mut V, label: BlankIdBuf| {
		vocabulary.insert_blank_id(label.as_blank_id_ref())
	};

	quads
		.into_iter()
		.map(move |quad| treeldr::vocab::loc_quad_from_rdf(quad, vocabulary, generate))
		.collect()
}

fn build_from_dataset<V: IndexedVocabulary + V>(
	vocabulary: &mut V,
	dataset: grdf::meta::BTreeDataset<Id, IriIndex, Object<Span>, GraphLabel, Span>,
) -> grdf::BTreeDataset<Id, IriIndex, StrippedObject, GraphLabel> {
	let mut context = BuildContext::new();
	let mut generator = rdf_types::generator::Blank::new_with_prefix("t".to_string());
	context.apply_built_in_definitions(vocabulary, &mut generator);

	dataset
		.declare(&mut (), &mut context, vocabulary, &mut generator)
		.unwrap();
	dataset
		.define(&mut (), &mut context, vocabulary, &mut generator)
		.unwrap();

	let model = context.build(vocabulary, &mut generator).unwrap();

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
				let input_dataset = parse_meta_nquads(&mut vocabulary, input);
				let output = build_from_dataset(&mut vocabulary, input_dataset);
				let expected_output = parse_nquads(&mut vocabulary, expected_output);

				for quad in output.quads() {
					println!("{} .", quad.with(&vocabulary))
				}

				assert!(output.is_isomorphic_to(&expected_output))
			}
			Self::Negative { input } => {
				let mut vocabulary = rdf_types::IndexVocabulary::<IriIndex, BlankIdIndex>::new();
				let input_dataset = parse_meta_nquads(&mut vocabulary, input);
				let output = build_from_dataset(&mut vocabulary, input_dataset);
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
					input: concat!("tests/build/", stringify!($id), "-in.nq"),
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
	p003
}

negative! {
	n001,
	n002,
	n003
}
