#[cfg(feature = "derive")]
#[test]
fn list_unordered() {
	#[derive(treeldr::SerializeLd)]
	#[tldr(set)]
	pub struct UnorderedList(Vec<String>);
}

#[cfg(feature = "derive")]
#[test]
fn list_ordered() {
	#[derive(treeldr::SerializeLd)]
	#[tldr(list)]
	pub struct UnorderedList(Vec<String>);
}

#[cfg(feature = "derive")]
#[test]
fn list_sized() {
	#[derive(treeldr::SerializeLd)]
	pub struct SizedList(String, String, String);
}
