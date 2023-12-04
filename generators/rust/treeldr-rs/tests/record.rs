#[cfg(feature = "derive")]
#[test]
fn record() {
	#[derive(treeldr::SerializeLd, treeldr::DeserializeLd)]
	#[tldr(prefix("ex" = "http://example.org/#"))]
	pub struct Record {
		#[tldr("ex:foo")]
		foo: String,
	}
}
