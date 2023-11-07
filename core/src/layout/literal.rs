pub mod data;
pub mod id;

pub use data::{DataLayout, UnitLayout, BooleanLayout, NumberLayout, ByteStringLayout, TextStringLayout, DataLayoutType, UnitLayoutType, BooleanLayoutType, NumberLayoutType, ByteStringLayoutType, TextStringLayoutType};
pub use id::IdLayout;

pub struct LiteralLayoutType;

pub enum LiteralLayout<R> {
	Data(DataLayout<R>),
	Id(IdLayout<R>),
}
