use treeldr_rust_macros::tldr;

#[tldr("modules/lexicon/tests/t02.lexicon.json", no_rdf)]
pub mod schema {
	#[prefix("lexicon:com")]
	pub mod com {}
}

#[test]
fn t02() {
	let _value = schema::com::Example::new("foo".to_string());
}
