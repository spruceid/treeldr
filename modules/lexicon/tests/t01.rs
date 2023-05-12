use treeldr_rust_macros::tldr;

#[tldr("modules/lexicon/tests/t01.lexicon.json", no_rdf)]
pub mod schema {
	#[prefix("lexicon:com.atproto")]
	pub mod atproto {}
}

#[test]
fn t01() {
	let _query = schema::atproto::identity::ResolveHandle {
		handle: Some("@hello".to_string()),
	};
}
