use std::fmt::Debug;

use iref::Iri;
use json_ld::{
	syntax::{Parse as ParseJson, TryFromJson},
	ContextLoader, Print, Process,
};
use locspan::{BorrowStripped, Span};
use rdf_types::{IndexVocabulary, IriVocabulary, VocabularyMut};
use static_iref::iri;
use treeldr::{BlankIdIndex, Id, IriIndex, TId};
use treeldr_build::Document;
use treeldr_syntax::Parse;

#[derive(Debug, Default)]
pub struct Options {
	rdf_type_to_layout_name: bool,
	flatten: bool,
	prefixes: Vec<(&'static str, Iri<'static>)>,
	context: Option<&'static str>,
}

impl Options {
	pub async fn load<V, L>(
		self,
		vocabulary: &mut V,
		loader: &mut L,
	) -> treeldr_json_ld_context::Options<Span>
	where
		V: Send + Sync + VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>,
		L: Send + Sync + ContextLoader<IriIndex, Span>,
		L::Context: Into<json_ld::syntax::context::Value<Span>>,
		L::ContextError: Debug,
	{
		let context = match self.context {
			Some(content) => {
				let json = json_ld::syntax::Value::parse_str(content, |span| span).unwrap();
				let json_context = json_ld::syntax::context::Value::try_from_json(json).unwrap();
				json_context
					.process(vocabulary, loader, None)
					.await
					.unwrap()
					.into_processed()
			}
			None => json_ld::Context::default(),
		};

		let prefixes = self
			.prefixes
			.iter()
			.map(|(prefix, iri)| (prefix.to_string(), vocabulary.insert(*iri)))
			.collect();

		treeldr_json_ld_context::Options {
			rdf_type_to_layout_name: self.rdf_type_to_layout_name,
			flatten: self.flatten,
			prefixes,
			context,
		}
	}
}

pub enum Test {
	Positive {
		input: &'static str,
		layouts: &'static [&'static str],
		expected_output: &'static str,
		options: Options,
	},
	Negative {
		input: &'static str,
		layouts: &'static [&'static str],
		options: Options,
	},
}

impl Test {
	async fn run(self) {
		match self {
			Self::Positive {
				input,
				layouts,
				expected_output,
				options,
			} => {
				let ast =
					treeldr_syntax::Document::parse_str(&input, |span| span).expect("parse error");
				let mut context = treeldr_build::Context::new();
				let mut vocabulary = IndexVocabulary::new();
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

				let layouts: Vec<_> = layouts
					.into_iter()
					.map(|iri| {
						let id: TId<treeldr::Layout> = TId::new(Id::Iri(
							vocabulary.get(Iri::from_str(iri).unwrap()).unwrap(),
						));

						model.require(id).unwrap();

						id
					})
					.collect();

				let mut loader = json_ld::NoLoader::<IriIndex, Span>::new();
				let options = options.load(&mut vocabulary, &mut loader).await;

				let output = treeldr_json_ld_context::generate(
					&mut vocabulary,
					&mut loader,
					&model,
					options,
					&layouts,
				)
				.await
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
			Self::Negative {
				input,
				layouts,
				options,
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

				let layouts: Vec<_> = layouts
					.into_iter()
					.map(|iri| {
						let id: TId<treeldr::Layout> = TId::new(Id::Iri(
							vocabulary.get(Iri::from_str(iri).unwrap()).unwrap(),
						));

						model.require(id).unwrap();

						id
					})
					.collect();

				let mut loader = json_ld::NoLoader::<IriIndex, Span>::new();
				let options = options.load(&mut vocabulary, &mut loader).await;

				let output = treeldr_json_ld_context::generate(
					&mut vocabulary,
					&mut loader,
					&model,
					options,
					&layouts,
				)
				.await
				.expect("unable to generate LD context");

				eprintln!("output:\n{}", output.pretty_print());
			}
		}
	}
}

macro_rules! positive {
	{ $($id:ident : [$($iri:literal),*] $({ $($option:ident: $value:expr),* })?),* } => {
		$(
			#[async_std::test]
			async fn $id () {
				Test::Positive {
					input: include_str!(concat!("generate_json_ld_context/", stringify!($id), "-in.tldr")),
					layouts: &[$($iri,)*],
					expected_output: include_str!(concat!("generate_json_ld_context/", stringify!($id), "-out.json")),
					options: Options {
						$($(
							$option: $value,
						)*)?
						..Default::default()
					}
				}.run().await
			}
		)*
	};
}

macro_rules! negative {
	{ $($id:ident : [$($iri:literal),*] $({ $($option:ident: $value:expr),* })?),* } => {
		$(
			#[async_std::test]
			#[should_panic]
			async fn $id () {
				Test::Negative {
					input: include_str!(concat!("generate_json_ld_context/", stringify!($id), ".tldr")),
					layouts: &[$($iri,)*],
					options: Options {
						$($(
							$option: $value,
						)*)?
						..Default::default()
					}
				}.run().await
			}
		)*
	};
}

positive! {
	p001: ["http://www.example.com/Foo"],
	p002: ["http://www.example.com/Foo"],
	p003: ["http://www.example.com/Foo"],
	p004: ["http://www.example.com/Foo", "http://www.example.com/Bar"],
	p005: ["http://www.example.com/Foo"] { rdf_type_to_layout_name: true },
	p006: ["http://www.example.com/Foo"] { rdf_type_to_layout_name: true },
	p007: ["http://www.example.com/Foo"] { rdf_type_to_layout_name: true },
	p008: ["http://www.example.com/Foo"],
	p009: ["http://www.example.com/Foo"],
	p010: ["http://www.example.com/Foo"],
	p011: ["http://www.example.com/Foo"],
	p012: ["http://www.example.com/Foo"] { context: Some(include_str!("generate_json_ld_context/p012-context.json")) },
	p013: ["http://www.example.com/Foo"] { rdf_type_to_layout_name: true, context: Some(include_str!("generate_json_ld_context/p013-context.json")) },
	p014: ["http://www.example.com/Foo"] { rdf_type_to_layout_name: true, context: Some(include_str!("generate_json_ld_context/p014-context.json")) },
	p015: ["http://www.example.com/Foo"] { rdf_type_to_layout_name: true, context: Some(include_str!("generate_json_ld_context/p015-context.json")) },
	p016: ["http://www.example.com/CustomCredential"] { rdf_type_to_layout_name: true, context: Some(include_str!("generate_json_ld_context/p016-context.json")) },
	p017: ["http://www.example.com/Foo"],
	p018: ["http://www.example.com/Foo"],
	p019: ["http://www.example.com/Foo"],
	p020: ["http://www.example.com/Bar"],
	p021: ["http://www.example.com/Foo"],
	p022: ["http://www.example.com/A"],
	p023: ["http://www.example.com/Foo"] { rdf_type_to_layout_name: true },
	p024: ["http://www.example.com/Foo"] { rdf_type_to_layout_name: true, flatten: true },
	p025: ["http://www.example.com/Bar"] { rdf_type_to_layout_name: true },
	p026: ["http://www.example.com/Foo"] { rdf_type_to_layout_name: true, flatten: true, prefixes: vec![("ex", iri!("http://www.example.com/"))] }
}

negative! {
	n001: ["http://www.example.com/Foo", "http://www.example.com/Bar"]
}
