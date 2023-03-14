use iref::Iri;
use json_ld::{syntax::Parse, Expand, NoLoader, Print, RdfQuads};
use locspan::{Meta, Span};
use rdf_types::Id;
use treeldr_rust_macros::tldr;
use treeldr_rust_prelude::{ld::import_quad, static_iref::iri, FromRdf, IntoJsonLd};
// use treeldr_rust_prelude::{
// 	json_ld::import_quad,
// 	rdf::{Subject, SubjectRef},
// 	static_iref::iri,
// 	FromRdf, IntoJsonLd,
// };

/// Schema types generated by TreeLDR.
#[tldr(
	"examples/xsd.tldr",
	"examples/schema.org.tldr",
	"modules/rust/gen/examples/vc.tldr",
	"modules/rust/gen/examples/rebase.tldr",
	"modules/rust/gen/examples/basic_post.tldr"
)]
pub mod schema {
	/// TLDR types.
	#[prefix("https://treeldr.org/")]
	pub mod tldr {}

	/// RDF Schema types.
	#[prefix("http://www.w3.org/2000/01/rdf-schema#")]
	pub mod rdfs {}

	/// XSD types.
	#[prefix("http://www.w3.org/2001/XMLSchema#")]
	pub mod xsd {}

	/// schema.org.
	#[prefix("https://schema.org/")]
	pub mod org {}

	/// Verifiable Credential schema.
	#[prefix("https://www.w3.org/2018/credentials#")]
	pub mod vc {}

	/// Rebase types.
	#[prefix("https://example.com/rebase/")]
	pub mod rebase {}

	/// Basic Post example types.
	#[prefix("https://example.com/example/")]
	pub mod basic_post {}
}

const VC_LD_CONTEXT_URL: Iri<'static> = iri!("https://www.w3.org/2018/credentials/v1");

// type JsonLd = json_ld::syntax::Value<Span>;
// type NoLoader = json_ld::loader::NoLoader<IriBuf, Span, JsonLd>;

#[async_std::main]
async fn main() {
	// Read JSON-LD file.
	let filename = "modules/rust/gen/examples/basic_post.jsonld";
	let input = std::fs::read_to_string(filename).unwrap();
	let json_ld = json_ld::syntax::Value::parse_str(&input, |span| span).expect("invalid JSON");

	// Expand JSON-LD.
	let expanded_json_ld = json_ld
		.expand(&mut NoLoader::<_, _, json_ld::syntax::Value<Span>>::new())
		.await
		.expect("expansion failed");

	// JSON-LD to RDF.
	let mut generator = rdf_types::generator::Blank::new().with_default_metadata();
	let dataset: grdf::HashDataset<_, _, _, _> = expanded_json_ld
		.rdf_quads(&mut generator, None)
		.map(import_quad)
		.collect();

	// RDF into schema generated from TreeLDR.
	let post = schema::basic_post::BasicPost::from_rdf(
		&mut (),
		&Id::Iri(iri!("https://example.com/#MyPost").to_owned()),
		dataset.default_graph(),
	)
	.expect("invalid post");

	// Wrap the post inside a VC.
	let mut vc = schema::basic_post::VerifiableBasicPost::new(chrono::Utc::now());
	vc.credential_subject = Some(post).into_iter().collect();
	vc.type_.extend([
		treeldr_rust_prelude::Id(Id::Iri(
			iri!("https://www.w3.org/2018/credentials#VerifiableCredential").to_owned(),
		)),
		treeldr_rust_prelude::Id(Id::Iri(
			iri!("https://example.com/example/VerifiableBasicPost").to_owned(),
		)),
	]);

	// Schema to JSON-LD.
	let mut json_ld_out = vc.into_json_ld(&());
	json_ld_out.as_object_mut().unwrap().insert(
		Meta("@context".into(), ()),
		Meta(VC_LD_CONTEXT_URL.as_str().into(), ()),
	);

	// Print the result.
	println!("{}", json_ld_out.pretty_print());
}
