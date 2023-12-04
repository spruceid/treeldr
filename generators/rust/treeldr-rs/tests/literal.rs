#[cfg(feature = "derive")]
#[test]
fn literal_unit() {
	#[derive(treeldr::SerializeLd)]
	pub struct Unit;
}

#[cfg(feature = "derive")]
#[test]
fn literal_boolean() {
	#[derive(treeldr::SerializeLd)]
	#[tldr(boolean)]
	pub struct Boolean(bool);
}

#[cfg(feature = "derive")]
#[test]
fn literal_i32() {
	#[derive(treeldr::SerializeLd)]
	#[tldr(prefix("xsd" = "http://www.w3.org/2001/XMLSchema"))]
	#[tldr(number, datatype("xsd:int"))]
	pub struct I32(i32);
}
