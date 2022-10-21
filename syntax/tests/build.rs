use contextual::WithContext;
use locspan::{Meta, Span};
use nquads_syntax::BlankIdBuf;
use rdf_types::VocabularyMut;
use static_iref::iri;
use std::path::Path;
use treeldr::{
	vocab::{GraphLabel, Id, StrippedObject},
	BlankIdIndex, IriIndex,
};

type BuildContext = treeldr_build::Context<Span, treeldr_syntax::build::Descriptions>;

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

fn parse_treeldr<P: AsRef<Path>>(
	vocabulary: &mut impl VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>,
	path: P,
) -> grdf::BTreeDataset<Id, IriIndex, StrippedObject, GraphLabel> {
	use treeldr_build::Document;
	use treeldr_syntax::Parse;

	let input = std::fs::read_to_string(path).expect("unable to read input file");
	let ast = treeldr_syntax::Document::parse_str(&input, |span| span).expect("parse error");
	let mut context = BuildContext::new();
	let mut generator = rdf_types::generator::Blank::new_with_prefix("t".to_string());
	context
		.apply_built_in_definitions(vocabulary, &mut generator)
		.unwrap();
	let mut local_context =
		treeldr_syntax::build::LocalContext::new(Some(iri!("http://www.example.com").into()));

	ast.declare(&mut local_context, &mut context, vocabulary, &mut generator)
		.expect("build error");
	ast.into_value()
		.relate(&mut local_context, &mut context, vocabulary, &mut generator)
		.expect("build error");

	let context = context
		.simplify(vocabulary, &mut generator)
		.expect("simplification failed");

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
			#[test]
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
			#[test]
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
	t001,
	t002,
	t003,
	t004,
	t005,
	t007,
	t008,
	t009,
	t010,
	t011,
	t012
}

negative! {
	e01,
	e02
}
