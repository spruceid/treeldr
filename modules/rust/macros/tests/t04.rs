// cargo run -p treeldr-rust-cli -- -i modules/rust/gen/tests/t04.tldr -m rdfs="http://www.w3.org/2000/01/rdf-schema#" -m xsd="http://www.w3.org/2001/XMLSchema#" -m example="https://example.com/example/" | rustfmt
use iref::IriBuf;
use json_ld::{syntax::Parse, JsonLdProcessor};
use locspan::Span;
use rdf_types::{Id, Term};
use treeldr_rust_macros::tldr;
use treeldr_rust_prelude::{ld::import_quad, static_iref::iri, FromRdf};

#[tldr]
pub mod base_schema {
	#[prefix("http://www.w3.org/2000/01/rdf-schema#")]
	pub mod rdfs {}

	#[prefix("http://www.w3.org/2001/XMLSchema#")]
	pub mod xsd {}
}

#[tldr("modules/rust/macros/tests/t03.tldr")]
pub mod schema {
	#[prefix("http://www.w3.org/2000/01/rdf-schema#")]
	pub use crate::base_schema::rdfs as rdf_syntax;

	#[prefix("http://www.w3.org/2001/XMLSchema#")]
	pub use crate::base_schema::xsd;

	#[prefix("https://example.com/")]
	pub mod test {}
}

#[async_std::test]
async fn t04() {
	let mut loader: json_ld::FsLoader<IriBuf, Span> =
		json_ld::FsLoader::new(|_, _, s| json_syntax::Value::parse_str(s, |span| span));
	loader.mount(iri!("https://example.com/").to_owned(), "tests/");

	let doc: json_ld::RemoteDocumentReference<IriBuf, Span, _> =
		json_ld::RemoteDocumentReference::Iri(iri!("https://example.com/t04.jsonld").to_owned());
	let mut generator = rdf_types::generator::Blank::new().with_default_metadata();
	let mut to_rdf = doc
		.to_rdf(&mut generator, &mut loader)
		.await
		.expect("expansion failed");
	let dataset: grdf::HashDataset<_, _, _, _> = to_rdf.quads().map(import_quad).collect();

	for triple in dataset.default_graph() {
		eprintln!("{} .", triple)
	}

	let value = schema::test::layout::Foo::from_rdf(
		&mut (),
		&Term::Id(Id::Iri(iri!("https://example.com/subject").to_owned())),
		dataset.default_graph(),
	)
	.expect("invalid value");

	assert_eq!(
		value,
		schema::test::layout::Foo {
			name: "Foo".to_string()
		}
	);
}
