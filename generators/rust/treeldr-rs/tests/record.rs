#[cfg(feature = "derive")]
#[test]
fn record() {
	#[derive(treeldr::SerializeLd)]
	#[tldr(prefix("ex" = "http://example.org/#"))]
	pub struct Record {
		#[tldr("ex:foo")]
		foo: String,
	}
}
