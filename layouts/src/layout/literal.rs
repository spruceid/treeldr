pub mod data;
pub mod id;

pub use data::{
	BooleanLayout, BooleanLayoutType, ByteStringLayout, ByteStringLayoutType, DataLayout,
	DataLayoutType, NumberLayout, NumberLayoutType, TextStringLayout, TextStringLayoutType,
	UnitLayout, UnitLayoutType,
};
pub use id::{IdLayout, IdLayoutType};

pub struct LiteralLayoutType;

#[derive(Debug)]
pub enum LiteralLayout<R> {
	Data(DataLayout<R>),
	Id(IdLayout<R>),
}
