#![cfg(feature = "unicode-segmentation")]
use treeldr_rust_macros::tldr;

#[tldr("modules/rust/macros/tests/t07.ttl")]
pub mod schema {
	#[prefix("http://www.w3.org/2000/01/rdf-schema#")]
	pub mod rdfs {}

	#[prefix("http://www.w3.org/2001/XMLSchema#")]
	pub mod xsd {}

	#[prefix("https://example.com/")]
	pub mod test {}
}

#[async_std::test]
async fn t07() {
	assert!(schema::test::layout::Layout::new("b".to_string()).is_err());
	assert!(schema::test::layout::Layout::new("aa".to_string()).is_err());
	assert!(schema::test::layout::Layout::new("aaa".to_string()).is_ok());
	assert!(schema::test::layout::Layout::new("aaaaaaaaaaaaaaaaaaaaa".to_string()).is_err());
}
