#[cfg(feature = "macros")]
#[test]
fn sum() {
	#[derive(treeldr::SerializeLd, treeldr::DeserializeLd)]
	#[tldr(prefix("ex" = "http://example.org/#"))]
	pub enum Sum {
		Foo(String),
		Bar(String),
	}
}
