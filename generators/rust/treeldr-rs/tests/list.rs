#[cfg(feature = "macros")]
#[test]
fn list_unordered() {
	#[derive(treeldr::SerializeLd, treeldr::DeserializeLd)]
	#[tldr(set)]
	pub struct UnorderedList(Vec<String>);
}

#[cfg(feature = "macros")]
#[test]
fn list_ordered() {
	#[derive(treeldr::SerializeLd, treeldr::DeserializeLd)]
	#[tldr(list)]
	pub struct UnorderedList(Vec<String>);
}

#[cfg(feature = "macros")]
#[test]
fn list_sized() {
	#[derive(treeldr::SerializeLd, treeldr::DeserializeLd)]
	pub struct SizedList(String, String, String);
}
