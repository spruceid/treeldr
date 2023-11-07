use std::collections::BTreeMap;

use treeldr::{
	layout::{ListLayoutType, ProductLayoutType, SumLayoutType, UnitLayoutType, BooleanLayoutType, NumberLayoutType, ByteStringLayoutType, TextStringLayoutType},
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
	Unit(Ref<R, UnitLayoutType>),

	/// Boolean value.
	Boolean(bool, Ref<R, BooleanLayoutType>),

	/// Any rational number.
	Number(Number, Ref<R, NumberLayoutType>),

	/// Byte string.
	ByteString(Vec<u8>, Ref<R, ByteStringLayoutType>),

	/// Text string.
	TextString(String, Ref<R, TextStringLayoutType>),
}

/// Typed value.
pub enum TypedValue<R> {
	Literal(TypedLiteral<R>),
	Variant(Box<Self>, Ref<R, SumLayoutType>, u32),
	Record(BTreeMap<String, Self>, Ref<R, ProductLayoutType>),
	List(Vec<Self>, Ref<R, ListLayoutType>),
}