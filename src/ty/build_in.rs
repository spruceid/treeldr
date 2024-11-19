#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BuiltInType {
	Any,
	Resource,
	Unit,
	Boolean,
	Number,
	Int,
	Uint,
	ByteString,
	TextString,
}
