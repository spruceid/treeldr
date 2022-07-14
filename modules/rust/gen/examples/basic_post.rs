use decoded_char::DecodedChars;
use grdf::Dataset;
use iref::{Iri, IriBuf};
use json_ld::Expand;
use json_syntax::{Parse, Print};
use locspan::Location;
use treeldr_rust_macros::tldr;
use treeldr_rust_prelude::{
	json_ld::import_quad, static_iref::iri, FromRdf, IntoJsonLd, SubjectRef,
};

#[tldr(
	"examples/xsd.tldr",
	"examples/schema.org.tldr",
	"modules/rust/gen/examples/vc.tldr",
	"modules/rust/gen/examples/rebase.tldr",
	"modules/rust/gen/examples/basic_post.tldr"
)]
pub mod schema {
	#[prefix("http://www.w3.org/2000/01/rdf-schema#")]
	pub mod rdfs {}

	#[prefix("http://www.w3.org/2001/XMLSchema#")]
	pub mod xsd {}

	#[prefix("https://www.w3.org/2018/credentials#")]
	pub mod vc {}

	#[prefix("https://example.com/rebase/")]
	pub mod rebase {}

	#[prefix("https://example.com/example/")]
	pub mod basic_post {}
}

const VC_LD_CONTEXT_URL: Iri<'static> = iri!("https://www.w3.org/2018/credentials/v1");

type LocalContext<'a> = json_ld::syntax::ContextEntry<Location<&'a str>>;
type Context<'a> = json_ld::Context<IriBuf, LocalContext<'a>>;
type JsonLd<'a> = json_ld::syntax::Value<LocalContext<'a>, Location<&'a str>>;
type Loader<'a> = json_ld::loader::NoLoader<JsonLd<'a>, Location<&'a str>>;

#[async_std::main]
async fn main() {
	// Read JSON-LD file.
	let filename = "examples/basic_post.jsonld";
	let input = std::fs::read_to_string(filename).unwrap();
	let json = json_syntax::Value::parse(filename, input.decoded_chars().map(infallible))
		.expect("parse error");
	let json_ld = json_ld::syntax::Value::try_from_json(json).expect("invalid JSON-LD");

	// Expand JSON-LD.
	let context = Context::default();
	let expanded_json_ld = json_ld
		.expand(&context, &mut Loader::new())
		.await
		.expect("expansion failed");

	// JSON-LD to RDF.
	let mut generator = json_ld::id::generator::Blank::new();
	let dataset: grdf::HashDataset<_, _, _, _> = expanded_json_ld
		.rdf_quads(&mut generator, None)
		.map(import_quad)
		.collect();

	// RDF into schema generated from TreeLDR.
	let post = schema::basic_post::BasicPost::from_rdf(
		SubjectRef::Iri(iri!("https://example.com/#MyPost")),
		dataset.default_graph(),
	)
	.expect("invalid post");

	// Wrap the post inside a VC.
	let mut vc = schema::basic_post::VerifiableBasicPost::default();
	vc.credential_subject = Some(post).into_iter().collect();

	// Schema to JSON-LD.
	let mut json_ld: json_ld::syntax::Value<json_ld::syntax::ContextEntry<()>, _> =
		vc.into_json_ld();
	json_ld
		.as_object_mut()
		.unwrap()
		.append_context(VC_LD_CONTEXT_URL.into());
	println!("{}", json_ld.pretty_print());
}

fn infallible<T>(t: T) -> Result<T, std::convert::Infallible> {
	Ok(t)
}
