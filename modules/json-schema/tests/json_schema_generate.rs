use iref::Iri;
use json_syntax::{BorrowUnordered, Parse as ParseJson, Print};
use rdf_types::IriVocabulary;
use static_iref::iri;
use treeldr::{Id, TId};
use treeldr_build::Document;
use treeldr_syntax::Parse;

pub enum Test {
	Positive {
		input: &'static str,
		layout: &'static str,
		expected_output: &'static str,
	},
	Negative {
		input: &'static str,
		layout: &'static str,
	},
}

impl Test {
	fn run(self) {
		match self {
			Self::Positive {
				input,
				layout,
				expected_output,
			} => {
				let ast =
					treeldr_syntax::Document::parse_str(&input, |span| span).expect("parse error");
				let mut context = treeldr_build::Context::new();
				let mut vocabulary = rdf_types::IndexVocabulary::new();
				let mut generator = rdf_types::generator::Blank::new();

				context.apply_built_in_definitions(&mut vocabulary, &mut generator);
				let mut local_context = treeldr_syntax::build::LocalContext::new(Some(
					iri!("http://www.example.com").into(),
				));

				ast.declare(
					&mut local_context,
					&mut context,
					&mut vocabulary,
					&mut generator,
				)
				.expect("build error");
				ast.into_value()
					.define(
						&mut local_context,
						&mut context,
						&mut vocabulary,
						&mut generator,
					)
					.expect("build error");

				let model = context
					.build(&mut vocabulary, &mut generator)
					.expect("build error");

				let layout_ref: TId<treeldr::Layout> = TId::new(Id::Iri(
					vocabulary.get(Iri::from_str(layout).unwrap()).unwrap(),
				));

				model.require(layout_ref).unwrap();

				let embedding = treeldr_json_schema::embedding::Configuration::default();

				let output = treeldr_json_schema::generate(
					&vocabulary,
					&model,
					&embedding,
					None,
					layout_ref,
				)
				.expect("unable to generate JSON Schema");

				let expected = json_syntax::Value::parse_str(expected_output, |_| ())
					.expect("invalid JSON")
					.into_value();

				let success = output.unordered() == expected.unordered();

				if !success {
					eprintln!(
						"output:\n{}\nexpected:\n{}",
						output.pretty_print(),
						expected.pretty_print()
					);
				}

				assert!(success)
			}
			Self::Negative { input, layout } => {
				let ast =
					treeldr_syntax::Document::parse_str(&input, |span| span).expect("parse error");
				let mut context = treeldr_build::Context::new();
				let mut vocabulary = rdf_types::IndexVocabulary::new();
				let mut generator = rdf_types::generator::Blank::new();

				context.apply_built_in_definitions(&mut vocabulary, &mut generator);
				let mut local_context = treeldr_syntax::build::LocalContext::new(Some(
					iri!("http://www.example.com").into(),
				));

				ast.declare(
					&mut local_context,
					&mut context,
					&mut vocabulary,
					&mut generator,
				)
				.expect("build error");
				ast.into_value()
					.define(
						&mut local_context,
						&mut context,
						&mut vocabulary,
						&mut generator,
					)
					.expect("build error");

				let model = context
					.build(&mut vocabulary, &mut generator)
					.expect("build error");

				let layout_ref: TId<treeldr::Layout> = TId::new(Id::Iri(
					vocabulary.get(Iri::from_str(layout).unwrap()).unwrap(),
				));

				model.require(layout_ref).unwrap();

				let embedding = treeldr_json_schema::embedding::Configuration::default();

				let output = treeldr_json_schema::generate(
					&vocabulary,
					&model,
					&embedding,
					None,
					layout_ref,
				)
				.expect("unable to generate JSON Schema");

				eprintln!("output:\n{}", output.pretty_print());
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
					input: include_str!(concat!("json_schema_generate/", stringify!($id), "-in.tldr")),
					layout: $iri,
					expected_output: include_str!(concat!("json_schema_generate/", stringify!($id), "-out.schema.json"))
				}.run()
			}
		)*
	};
}

positive! {
	p001: "http://www.example.com/Foo",
	p002: "http://www.example.com/Foo",
	p003: "http://www.example.com/Foo",
	p004: "http://www.example.com/Foo"
}
