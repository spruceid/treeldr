use iref::Iri;
use rdf_types::IriVocabulary;
use static_iref::iri;
use treeldr::Id;
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

				context
					.apply_built_in_definitions(&mut vocabulary, &mut generator)
					.unwrap();
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
					.relate(
						&mut local_context,
						&mut context,
						&mut vocabulary,
						&mut generator,
					)
					.expect("build error");

				let context = context
					.simplify(&mut vocabulary, &mut generator)
					.expect("simplification failed");

				let model = context
					.build(&mut vocabulary, &mut generator)
					.expect("build error");

				let layout_ref = model
					.require_layout(Id::Iri(
						vocabulary.get(Iri::from_str(layout).unwrap()).unwrap(),
					))
					.unwrap();

				let embedding = treeldr_json_schema::embedding::Configuration::default();

				let output = treeldr_json_schema::generate(
					&vocabulary,
					&model,
					&embedding,
					None,
					layout_ref,
				)
				.expect("unable to generate JSON Schema");

				let expected: serde_json::Value =
					serde_json::from_str(expected_output).expect("invalid JSON");

				let success = output == expected;

				if !success {
					eprintln!(
						"output:\n{}\nexpected:\n{}",
						serde_json::to_string_pretty(&output).unwrap(),
						serde_json::to_string_pretty(&expected).unwrap()
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

				context
					.apply_built_in_definitions(&mut vocabulary, &mut generator)
					.unwrap();
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
					.relate(
						&mut local_context,
						&mut context,
						&mut vocabulary,
						&mut generator,
					)
					.expect("build error");

				let context = context
					.simplify(&mut vocabulary, &mut generator)
					.expect("simplification failed");

				let model = context
					.build(&mut vocabulary, &mut generator)
					.expect("build error");

				let layout_ref = model
					.require_layout(Id::Iri(
						vocabulary.get(Iri::from_str(layout).unwrap()).unwrap(),
					))
					.unwrap();

				let embedding = treeldr_json_schema::embedding::Configuration::default();

				let output = treeldr_json_schema::generate(
					&vocabulary,
					&model,
					&embedding,
					None,
					layout_ref,
				)
				.expect("unable to generate JSON Schema");

				eprintln!(
					"output:\n{}",
					serde_json::to_string_pretty(&output).unwrap()
				);
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
					input: include_str!(concat!("generate/", stringify!($id), "-in.tldr")),
					layout: $iri,
					expected_output: include_str!(concat!("generate/", stringify!($id), "-out.schema.json"))
				}.run()
			}
		)*
	};
}

positive! {
	t01: "http://www.example.com/Foo"
}
