use treeldr_rust_macros::tldr;

#[tldr("modules/rust/macros/tests/t08.ttl")]
pub mod schema {
	#[prefix("http://www.w3.org/2000/01/rdf-schema#")]
	pub mod rdfs {}

	#[prefix("http://www.w3.org/2001/XMLSchema#")]
	pub mod xsd {}

	#[prefix("https://example.com/")]
	pub mod test {}
}

#[async_std::test]
async fn t08() {
	schema::test::layout::Layout::new("any".to_string());
}
