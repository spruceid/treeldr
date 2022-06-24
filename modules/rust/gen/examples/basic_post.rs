use treeldr_rust_macros::tldr;

#[tldr(
	"examples/xsd.tldr",
	"examples/schema.org.tldr",
	"modules/rust/gen/examples/vc.tldr",
	"modules/rust/gen/examples/rebase.tldr",
	"modules/rust/gen/examples/basic_post.tldr"
)]
mod schema {
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

fn main() {
	let mut post =
		schema::basic_post::BasicPost::new(iref::IriBuf::new("https://example.com/#MyPost").unwrap());
	post.title = Some("Title".to_string());
	post.body = Some("Lorem ipsum dolor sit amet, consectetur adipiscing elit.".to_string());

	let mut vc = schema::basic_post::VerifiableBasicPost::default();
	vc.credential_subject = Some(post);

	println!("Created post `{}`", vc.credential_subject.as_ref().unwrap().id)
}
