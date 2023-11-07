pub mod data;
pub mod id;

pub use data::{
	BooleanLayout, ByteStringLayout, DataLayout, NumberLayout, TextStringLayout, UnitLayout,
};
pub use id::{IdLayout, IdLayoutType};

pub struct LiteralLayoutType;

pub enum LiteralLayout<R> {
	Data(DataLayout<R>),
	Id(IdLayout<R>),
}

impl<R: Clone> LiteralLayout<R> {
	pub fn build(&self) -> treeldr::layout::LiteralLayout<R> {
		match self {
			Self::Data(layout) => treeldr::layout::LiteralLayout::Data(layout.build()),
			Self::Id(layout) => treeldr::layout::LiteralLayout::Id(layout.build()),
		}
	}
}
