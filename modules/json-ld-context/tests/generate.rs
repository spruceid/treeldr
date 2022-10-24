use iref::Iri;
use json_ld::{
	syntax::{Parse as ParseJson, TryFromJson},
	Print,
};
use locspan::BorrowStripped;
use rdf_types::IriVocabulary;
use static_iref::iri;
use treeldr::Id;
use treeldr_build::Document;
use treeldr_syntax::Parse;

pub enum Test {
	Positive {
		input: &'static str,
		layouts: &'static [&'static str],
		expected_output: &'static str,
	},
	Negative {
		input: &'static str,
		layouts: &'static [&'static str],
	},
}

impl Test {
	fn run(self) {
		match self {
			Self::Positive {
				input,
				layouts,
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

				let layouts: Vec<_> = layouts
					.into_iter()
					.map(|iri| {
						model
							.require_layout(Id::Iri(
								vocabulary.get(Iri::from_str(iri).unwrap()).unwrap(),
							))
							.unwrap()
					})
					.collect();

				let output = treeldr_json_ld_context::generate(
					&vocabulary,
					&model,
					treeldr_json_ld_context::Options::default(),
					&layouts,
				)
				.expect("unable to generate LD context");

				let expected = json_ld::syntax::Value::parse_str(expected_output, |_| ())
					.expect("invalid JSON");
				let expected = json_ld::syntax::context::Value::try_from_json(expected)
					.expect("invalid JSON-LD context")
					.into_value();

				let success = output.stripped() == expected.stripped();

				if !success {
					eprintln!(
						"output:\n{}\nexpected:\n{}",
						output.pretty_print(),
						expected.pretty_print()
					);
				}

				assert!(success)
			}
			Self::Negative { input, layouts } => {
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

				let layouts: Vec<_> = layouts
					.into_iter()
					.map(|iri| {
						model
							.require_layout(Id::Iri(
								vocabulary.get(Iri::from_str(iri).unwrap()).unwrap(),
							))
							.unwrap()
					})
					.collect();

				let output = treeldr_json_ld_context::generate(
					&vocabulary,
					&model,
					treeldr_json_ld_context::Options::default(),
					&layouts,
				)
				.expect("unable to generate LD context");

				eprintln!("output:\n{}", output.pretty_print());
			}
		}
	}
}

macro_rules! positive {
	{ $($id:ident : [$($iri:literal),*]),* } => {
		$(
			#[test]
			fn $id () {
				Test::Positive {
					input: include_str!(concat!("generate/", stringify!($id), "-in.tldr")),
					layouts: &[$($iri,)*],
					expected_output: include_str!(concat!("generate/", stringify!($id), "-out.json"))
				}.run()
			}
		)*
	};
}

macro_rules! negative {
	{ $($id:ident : [$($iri:literal),*]),* } => {
		$(
			#[test]
			#[should_panic]
			fn $id () {
				Test::Negative {
					input: include_str!(concat!("generate/", stringify!($id), ".tldr")),
					layouts: &[$($iri,)*]
				}.run()
			}
		)*
	};
}

positive! {
	t01: ["http://www.example.com/Foo"],
	t02: ["http://www.example.com/Foo"],
	t03: ["http://www.example.com/Foo"],
	t04: ["http://www.example.com/Foo", "http://www.example.com/Bar"]
}

negative! {
	e01: ["http://www.example.com/Foo", "http://www.example.com/Bar"]
}
