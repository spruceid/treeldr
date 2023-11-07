use treeldr::{Dataset, Pattern};

pub use treeldr::layout::{
	BooleanLayout, BooleanLayoutType, ByteStringLayout, ByteStringLayoutType, NumberLayout,
	NumberLayoutType, TextStringLayoutType, UnitLayout, UnitLayoutType,
};

use crate::RegExp;

/// Data layout.
#[derive(Clone)]
pub enum DataLayout<R> {
	Unit(UnitLayout<R>),
	Boolean(BooleanLayout<R>),
	Number(NumberLayout<R>),
	ByteString(ByteStringLayout<R>),
	TextString(TextStringLayout<R>),
}

impl<R: Clone> DataLayout<R> {
	pub fn build(&self) -> treeldr::layout::DataLayout<R> {
		match self {
			Self::Unit(layout) => treeldr::layout::DataLayout::Unit(layout.clone()),
			Self::Boolean(layout) => treeldr::layout::DataLayout::Boolean(layout.clone()),
			Self::Number(layout) => treeldr::layout::DataLayout::Number(layout.clone()),
			Self::ByteString(layout) => treeldr::layout::DataLayout::ByteString(layout.clone()),
			Self::TextString(layout) => treeldr::layout::DataLayout::TextString(layout.build()),
		}
	}
}

#[derive(Clone)]
pub struct TextStringLayout<R> {
	pub input: u32,

	pub intro: u32,

	pub pattern: Option<RegExp>,

	pub dataset: Dataset<R>,

	pub resource: Pattern<R>,

	pub type_: R,
}

impl<R: Clone> TextStringLayout<R> {
	pub fn build(&self) -> treeldr::layout::TextStringLayout<R> {
		treeldr::layout::TextStringLayout {
			input: self.input,
			intro: self.intro,
			pattern: self.pattern.as_ref().map(|e| e.build()),
			dataset: self.dataset.clone(),
			resource: self.resource.clone(),
			type_: self.type_.clone(),
		}
	}
}
