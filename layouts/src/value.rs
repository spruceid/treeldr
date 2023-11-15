use core::fmt;
use std::collections::BTreeMap;

use num_rational::BigRational;

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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Number(BigRational);

impl Number {
	pub fn new(rational: BigRational) -> Self {
		Self(rational)
	}

	pub fn as_big_rational(&self) -> &BigRational {
		&self.0
	}

	pub fn into_big_rational(self) -> BigRational {
		self.0
	}
}

impl From<BigRational> for Number {
	fn from(value: BigRational) -> Self {
		Self(value)
	}
}

impl From<Number> for BigRational {
	fn from(value: Number) -> Self {
		value.into_big_rational()
	}
}

macro_rules! number_from_integer {
	($($ty:ty),*) => {
		$(
			impl From<$ty> for Number {
				fn from(value: $ty) -> Self {
					Self(BigRational::from_integer(value.into()))
				}
			}
		)*
	};
}

#[derive(Debug, thiserror::Error)]
#[error("non finite float value `{0}`")]
pub struct NonFiniteFloat<T>(pub T);

macro_rules! number_from_float {
	($($ty:ty),*) => {
		$(
			impl TryFrom<$ty> for Number {
				type Error = NonFiniteFloat<$ty>;

				fn try_from(value: $ty) -> Result<Self, Self::Error> {
					match BigRational::from_float(value) {
						Some(v) => Ok(Self(v)),
						None => Err(NonFiniteFloat(value))
					}
				}
			}
		)*
	};
}

number_from_integer!(u8, u16, u32, u64, i8, i16, i32, i64);
number_from_float!(f32, f64);

impl fmt::Display for Number {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.0.fmt(f)
	}
}

impl From<serde_json::Number> for Number {
	fn from(value: serde_json::Number) -> Self {
		Self(
			<xsd_types::Decimal as xsd_types::ParseRdf>::parse_rdf(&value.to_string())
				.ok()
				.unwrap()
				.into(),
		)
	}
}

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

impl From<serde_json::Value> for Value {
	fn from(value: serde_json::Value) -> Self {
		match value {
			serde_json::Value::Null => Self::Literal(Literal::Unit),
			serde_json::Value::Bool(b) => Self::Literal(Literal::Boolean(b)),
			serde_json::Value::Number(n) => Self::Literal(Literal::Number(n.into())),
			serde_json::Value::String(s) => Self::Literal(Literal::TextString(s)),
			serde_json::Value::Array(items) => {
				Self::List(items.into_iter().map(Into::into).collect())
			}
			serde_json::Value::Object(entries) => Self::Record(
				entries
					.into_iter()
					.map(|(key, value)| (key, value.into()))
					.collect(),
			),
		}
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
