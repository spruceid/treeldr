use contextual::WithContext;
use iref::Iri;
use json_syntax::Parse;
use locspan::Meta;
use rdf_types::{BlankIdBuf, IriVocabularyMut, VocabularyMut};
use treeldr::{
	to_rdf::ToRdf,
	vocab::{GraphLabel, Id, StrippedObject},
	BlankIdIndex, IriIndex,
};
use treeldr_build::Context;

fn parse_nquads<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
	vocabulary: &mut V,
	content: &str,
) -> grdf::BTreeDataset<Id, IriIndex, StrippedObject, GraphLabel> {
	use nquads_syntax::{Document, Parse};

	let Meta(quads, _) = Document::parse_str(content, |span| span).expect("parse error");

	let generate = move |vocabulary: &mut V, label: BlankIdBuf| {
		vocabulary.insert_blank_id(label.as_blank_id_ref())
	};

	quads
		.into_iter()
		.map(move |quad| treeldr::vocab::stripped_loc_quad_from_rdf(quad, vocabulary, generate))
		.collect()
}

fn import_json_schema<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
	vocabulary: &mut V,
	content: &str,
) -> (
	grdf::BTreeDataset<Id, IriIndex, StrippedObject, GraphLabel>,
	Id,
) {
	let json = json_syntax::Value::parse_str(content, |_| ())
		.expect("invalid JSON")
		.into_value();
	let input = json_syntax::from_value(json).expect("invalid JSON Schema");

	let mut context: Context<()> = Context::new();
	let mut generator = rdf_types::generator::Blank::new_with_prefix("t".to_string());
	context.apply_built_in_definitions(vocabulary, &mut generator);
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

// use iref::Iri;
// use json_ld::{
// 	syntax::{Parse as ParseJson, TryFromJson},
// 	Print,
// };
// use locspan::BorrowStripped;
// use rdf_types::IriVocabulary;
// use static_iref::iri;
// use treeldr::Id;
// use treeldr_build::Document;
// use treeldr_json_ld_context::Options;
// use treeldr_syntax::Parse;

pub enum Test {
	Positive {
		input: &'static str,
		expected_iri: &'static str,
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
				expected_iri,
				expected_output,
			} => {
				let mut vocabulary = rdf_types::IndexVocabulary::<IriIndex, BlankIdIndex>::new();
				let expected_id = Id::Iri(vocabulary.insert(Iri::new(expected_iri).unwrap()));

				let (output, id) = import_json_schema(&mut vocabulary, input);
				let expected_output = parse_nquads(&mut vocabulary, expected_output);

				for quad in output.quads() {
					eprintln!("{} .", quad.with(&vocabulary))
				}

				assert!(output.is_isomorphic_to(&expected_output));
				assert_eq!(id, expected_id)
			}
			Self::Negative { input } => {
				let mut vocabulary = rdf_types::IndexVocabulary::<IriIndex, BlankIdIndex>::new();
				let (output, _) = import_json_schema(&mut vocabulary, input);

				for quad in output.quads() {
					eprintln!("{} .", quad.with(&vocabulary))
				}
			}
		}
	}
}

macro_rules! positive {
	{ $($id:ident : $iri:literal),* } => {
		$(
			#[test]
			fn $id () {
				Test::Positive {
					input: include_str!(concat!("json_schema_import/", stringify!($id), "-in.json")),
					expected_iri: $iri,
					expected_output: include_str!(concat!("json_schema_import/", stringify!($id), "-out.nq"))
				}.run()
			}
		)*
	};
}

positive! {
	p001: "https://treeldr.org/String",
	p002: "https://example.com/product.schema.json",
	p003: "https://example.com/product.schema.json",
	p004: "https://example.com/product.schema.json",
	p005: "https://example.com/product.schema.json",
	p006: "https://example.com/foo.schema.json"
}
