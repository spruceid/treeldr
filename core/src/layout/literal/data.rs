use crate::{utils::DetAutomaton, Dataset, Pattern};

pub struct DataLayoutType;

pub struct UnitLayoutType;

pub struct BooleanLayoutType;

pub struct NumberLayoutType;

pub struct ByteStringLayoutType;

pub struct TextStringLayoutType;

/// Data layout.
pub enum DataLayout<R> {
	Unit(UnitLayout<R>),
	Boolean(BooleanLayout<R>),
	Number(NumberLayout<R>),
	ByteString(ByteStringLayout<R>),
	TextString(TextStringLayout<R>),
}

pub struct UnitLayout<R> {
	pub input: u32,

	pub intro: u32,

	pub dataset: Dataset<R>,
}

pub struct BooleanLayout<R> {
	pub input: u32,

	pub intro: u32,

	pub dataset: Dataset<R>,

	pub resource: Pattern<R>,

	pub type_: R,
}

pub struct NumberLayout<R> {
	pub input: u32,

	pub intro: u32,

	pub dataset: Dataset<R>,

	pub resource: Pattern<R>,

	pub type_: R,
}

pub struct ByteStringLayout<R> {
	pub input: u32,

	pub intro: u32,

	pub dataset: Dataset<R>,

	pub resource: Pattern<R>,

	pub type_: R,
}

pub struct TextStringLayout<R> {
	pub input: u32,

	pub intro: u32,

	pub automaton: Option<DetAutomaton<usize>>,

	pub dataset: Dataset<R>,

	pub resource: Pattern<R>,

	pub type_: R,
}
