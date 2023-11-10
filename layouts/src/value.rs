use std::collections::BTreeMap;

use crate::{
	layout::{
		BooleanLayoutType, ByteStringLayoutType, IdLayoutType, ListLayoutType, NumberLayoutType,
		ProductLayoutType, SumLayoutType, TextStringLayoutType, UnitLayoutType,
	},
	Ref,
};

pub mod de;
pub mod ser;

/// Rational number.
pub type Number = num_rational::BigRational;

/// Literal value.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value {
	Literal(Literal),
	Record(BTreeMap<String, Self>),
	List(Vec<Self>),
}

impl Value {
	pub fn unit() -> Self {
		Self::Literal(Literal::Unit)
	}
}

pub enum TypedLiteral<R = rdf_types::Term> {
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

impl<R> TypedLiteral<R> {
	pub fn into_untyped(self) -> Literal {
		match self {
			Self::Unit(_) => Literal::Unit,
			Self::Boolean(b, _) => Literal::Boolean(b),
			Self::Number(n, _) => Literal::Number(n),
			Self::ByteString(s, _) => Literal::ByteString(s),
			Self::TextString(s, _) => Literal::TextString(s),
			Self::Id(i, _) => Literal::TextString(i),
		}
	}
}

/// Typed value.
pub enum TypedValue<R = rdf_types::Term> {
	Literal(TypedLiteral<R>),
	Variant(Box<Self>, Ref<SumLayoutType, R>, u32),
	Record(BTreeMap<String, Self>, Ref<ProductLayoutType, R>),
	List(Vec<Self>, Ref<ListLayoutType, R>),
	Always(Value),
}

impl<R> TypedValue<R> {
	pub fn into_untyped(self) -> Value {
		match self {
			Self::Literal(l) => Value::Literal(l.into_untyped()),
			Self::Variant(value, _, _) => value.into_untyped(),
			Self::Record(map, _) => Value::Record(
				map.into_iter()
					.map(|(k, v)| (k, v.into_untyped()))
					.collect(),
			),
			Self::List(items, _) => {
				Value::List(items.into_iter().map(TypedValue::into_untyped).collect())
			}
			Self::Always(value) => value,
		}
	}
}
