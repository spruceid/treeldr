use treeldr_rust_macros::tldr;

#[tldr("modules/lexicon/tests/t03.lexicon.json", no_rdf)]
pub mod schema {
	#[prefix("lexicon:com")]
	pub mod com {}
}

#[test]
fn t02() {
	let value = schema::com::Example::default();
	assert_eq!(value, schema::com::Example::new("foo".to_string()).unwrap());

	let value = schema::com::example::Integer::default();
	assert_eq!(value, schema::com::example::Integer::new(12.into()));

	let value = schema::com::example::Boolean::default();
	assert_eq!(value, schema::com::example::Boolean::new(true));
}
