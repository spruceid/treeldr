#[cfg(feature = "derive")]
#[test]
fn sum() {
	#[derive(treeldr::SerializeLd)]
	#[tldr(prefix("ex" = "http://example.org/#"))]
	pub enum Sum {
		Foo(String),
		Bar(String),
	}
}
