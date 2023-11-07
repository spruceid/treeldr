use std::collections::BTreeMap;

use crate::{
	layout::{
		BooleanLayoutType, ByteStringLayoutType, IdLayoutType, ListLayoutType, NumberLayoutType,
		ProductLayoutType, SumLayoutType, TextStringLayoutType, UnitLayoutType,
	},
	Ref,
};

/// Rational number.
pub type Number = num_rational::BigRational;

/// Literal value.
pub enum Literal {
	/// Unit.
	Unit,

	/// Boolean value.
	Boolean(bool),

	/// Any rational number.
	Number(Number),

	/// Byte string.
	ByteString(Vec<u8>),

	/// Text string.
	TextString(String),
}

/// Untyped value.
pub enum Value {
	Literal(Literal),
	Record(BTreeMap<String, Self>),
	List(Vec<Self>),
}

pub enum TypedLiteral<R> {
	/// Unit.
	Unit(Ref<UnitLayoutType, R>),

	/// Boolean value.
	Boolean(bool, Ref<BooleanLayoutType, R>),

	/// Any rational number.
	Number(Number, Ref<NumberLayoutType, R>),

	/// Byte string.
	ByteString(Vec<u8>, Ref<ByteStringLayoutType, R>),

	/// Text string.
	TextString(String, Ref<TextStringLayoutType, R>),

	/// Identifier.
	Id(String, Ref<IdLayoutType, R>),
}

/// Typed value.
pub enum TypedValue<R> {
	Literal(TypedLiteral<R>),
	Variant(Box<Self>, Ref<SumLayoutType, R>, u32),
	Record(BTreeMap<String, Self>, Ref<ProductLayoutType, R>),
	List(Vec<Self>, Ref<ListLayoutType, R>),
}
