use crate::{utils::DetAutomaton, Dataset, Pattern};

pub struct DataLayoutType;

pub struct UnitLayoutType;

pub struct BooleanLayoutType;

pub struct NumberLayoutType;

pub struct ByteStringLayoutType;

pub struct TextStringLayoutType;

/// Data layout.
#[derive(Clone)]
pub enum DataLayout<R> {
	Unit(UnitLayout<R>),
	Boolean(BooleanLayout<R>),
	Number(NumberLayout<R>),
	ByteString(ByteStringLayout<R>),
	TextString(TextStringLayout<R>),
}

#[derive(Clone)]
pub struct UnitLayout<R> {
	pub input: u32,

	pub intro: u32,

	pub dataset: Dataset<R>,
}

#[derive(Clone)]
pub struct BooleanLayout<R> {
	pub input: u32,

	pub intro: u32,

	pub dataset: Dataset<R>,

	pub resource: Pattern<R>,

	pub datatype: R,
}

#[derive(Clone)]
pub struct NumberLayout<R> {
	pub input: u32,

	pub intro: u32,

	pub dataset: Dataset<R>,

	pub resource: Pattern<R>,

	pub datatype: R,
}

#[derive(Clone)]
pub struct ByteStringLayout<R> {
	pub input: u32,

	pub intro: u32,

	pub dataset: Dataset<R>,

	pub resource: Pattern<R>,

	pub datatype: R,
}

#[derive(Clone)]
pub struct TextStringLayout<R> {
	pub input: u32,

	pub intro: u32,

	pub pattern: Option<DetAutomaton<usize>>,

	pub dataset: Dataset<R>,

	pub resource: Pattern<R>,

	pub datatype: R,
}
