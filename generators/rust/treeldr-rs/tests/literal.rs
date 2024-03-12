#[cfg(feature = "macros")]
#[test]
fn literal_unit() {
	#[derive(treeldr::SerializeLd, treeldr::DeserializeLd)]
	pub struct Unit;
}

#[cfg(feature = "macros")]
#[test]
fn literal_boolean() {
	#[derive(treeldr::SerializeLd, treeldr::DeserializeLd)]
	#[tldr(boolean)]
	pub struct Boolean(bool);
}

#[cfg(feature = "macros")]
#[test]
fn literal_i32() {
	#[derive(treeldr::SerializeLd, treeldr::DeserializeLd)]
	#[tldr(prefix("xsd" = "http://www.w3.org/2001/XMLSchema"))]
	#[tldr(number, datatype("xsd:int"))]
	pub struct I32(i32);
}

#[cfg(feature = "macros")]
#[test]
fn literal_string() {
	#[derive(treeldr::SerializeLd, treeldr::DeserializeLd)]
	#[tldr(prefix("xsd" = "http://www.w3.org/2001/XMLSchema"))]
	#[tldr(number, datatype("xsd:string"))]
	pub struct TestString(String);
}
