use crate::{Dataset, Pattern};

pub use crate::layout::{BooleanLayout, ByteStringLayout, NumberLayout, UnitLayout};

use crate::abs::RegExp;

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
	pub fn build(&self) -> crate::layout::DataLayout<R> {
		match self {
			Self::Unit(layout) => crate::layout::DataLayout::Unit(layout.clone()),
			Self::Boolean(layout) => crate::layout::DataLayout::Boolean(layout.clone()),
			Self::Number(layout) => crate::layout::DataLayout::Number(layout.clone()),
			Self::ByteString(layout) => crate::layout::DataLayout::ByteString(layout.clone()),
			Self::TextString(layout) => crate::layout::DataLayout::TextString(layout.build()),
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

	pub datatype: R,
}

impl<R: Clone> TextStringLayout<R> {
	pub fn build(&self) -> crate::layout::TextStringLayout<R> {
		crate::layout::TextStringLayout {
			input: self.input,
			intro: self.intro,
			pattern: self.pattern.as_ref().map(|e| e.build()),
			dataset: self.dataset.clone(),
			resource: self.resource.clone(),
			datatype: self.datatype.clone(),
		}
	}
}
