#[cfg(feature = "derive")]
#[test]
fn id() {
	#[derive(treeldr::SerializeLd)]
	#[tldr(id)]
	pub struct Id(String);
}