// cargo run -p treeldr-rust-cli -- -i modules/rust/gen/tests/t01.tldr -m rdfs="http://www.w3.org/2000/01/rdf-schema#" -m xsd="http://www.w3.org/2001/XMLSchema#" -m example="https://example.com/" | rustfmt
use iref::IriBuf;
use json_ld::{syntax::Parse, JsonLdProcessor};
use locspan::Span;
use rdf_types::{Id, Term};
use treeldr_rust_macros::tldr;
use treeldr_rust_prelude::{ld::import_quad, static_iref::iri, FromRdf};

#[tldr("modules/rust/macros/tests/t01.tldr")]
pub mod schema {
	#[prefix("https://treeldr.org/")]
	pub mod tldr {}

	#[prefix("http://www.w3.org/2000/01/rdf-schema#")]
	pub mod rdfs {}

	#[prefix("http://www.w3.org/2001/XMLSchema#")]
	pub mod xsd {}

	#[prefix("https://example.com/")]
	pub mod test {}
}

#[async_std::test]
async fn t01() {
	let mut loader: json_ld::FsLoader<IriBuf, Span> =
		json_ld::FsLoader::new(|_, _, s| json_syntax::Value::parse_str(s, |span| span));
	loader.mount(iri!("https://example.com/").to_owned(), "tests/");

	let doc: json_ld::RemoteDocumentReference<IriBuf, Span, _> =
		json_ld::RemoteDocumentReference::Iri(iri!("https://example.com/t01.jsonld").to_owned());
	let mut generator = rdf_types::generator::Blank::new().with_default_metadata();
	let mut to_rdf = doc
		.to_rdf(&mut generator, &mut loader)
		.await
		.expect("expansion failed");
	let dataset: grdf::HashDataset<_, _, _, _> = to_rdf.quads().map(import_quad).collect();

	for triple in dataset.default_graph() {
		eprintln!("{} .", triple)
	}

	let value = schema::test::layout::Enum::from_rdf(
		&mut (),
		&Term::Id(Id::Iri(iri!("https://example.com/subject").to_owned())),
		dataset.default_graph(),
	)
	.expect("invalid value");

	assert_eq!(
		value,
		schema::test::layout::Enum::A(schema::test::layout::A {
			a: Some("foo".to_string())
		})
	);
}
