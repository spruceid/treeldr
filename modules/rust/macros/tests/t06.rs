use treeldr_rust_macros::tldr;

#[tldr("modules/rust/macros/tests/t06.ttl")]
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
async fn t06() {
	assert!(schema::test::layout::Layout::new(0.into()).is_err());
	assert!(schema::test::layout::Layout::new(12.into()).is_ok());
}
