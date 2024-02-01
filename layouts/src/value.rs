use core::fmt;
use std::{collections::BTreeMap, str::FromStr};

use lazy_static::lazy_static;
use num_bigint::{BigInt, Sign};
use num_rational::BigRational;
use num_traits::{Signed, ToPrimitive, Zero};

use crate::{
	layout::{
		BooleanLayoutType, ByteStringLayoutType, IdLayoutType, LayoutType, ListLayoutType,
		NumberLayoutType, ProductLayoutType, SumLayoutType, TextStringLayoutType, UnitLayoutType,
	},
	Ref,
};

pub mod de;
pub mod ser;

#[cfg(feature = "cbor")]
mod cbor;

lazy_static! {
	static ref TEN: BigInt = 10u32.into();
}

/// Rational number.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Number(BigRational);

impl Number {
	pub fn new(rational: BigRational) -> Self {
		Self(rational)
	}

	pub fn is_integer(&self) -> bool {
		self.0.is_integer()
	}

	pub fn as_integer(&self) -> Option<&BigInt> {
		if self.0.is_integer() {
			Some(self.0.numer())
		} else {
			None
		}
	}

	pub fn as_big_rational(&self) -> &BigRational {
		&self.0
	}

	pub fn into_big_rational(self) -> BigRational {
		self.0
	}

	pub fn as_native(&self) -> NativeNumber {
		if self.0.is_integer() {
			let n = self.0.numer();
			match n.sign() {
				Sign::Minus => {
					if n.bits() < 64 {
						let unsigned = n.iter_u64_digits().next().unwrap() as i64;
						NativeNumber::I64(-unsigned)
					} else {
						NativeNumber::F64(self.0.to_f64().unwrap())
					}
				}
				Sign::NoSign | Sign::Plus => {
					if n.bits() <= 64 {
						NativeNumber::U64(n.iter_u64_digits().next().unwrap())
					} else {
						NativeNumber::F64(self.0.to_f64().unwrap())
					}
				}
			}
		} else {
			NativeNumber::F64(self.0.to_f64().unwrap())
		}
	}

	pub fn to_f64(&self) -> f64 {
		self.0.to_f64().unwrap()
	}

	/// Returns the decimal representation of this number, if there is one.
	pub fn decimal_representation(&self) -> Option<String> {
		use std::fmt::Write;

		let mut fraction = String::new();
		let mut map = std::collections::HashMap::new();

		let mut rem = if self.0.is_negative() {
			-self.0.numer()
		} else {
			self.0.numer().clone()
		};

		rem %= self.0.denom();
		while !rem.is_zero() && !map.contains_key(&rem) {
			map.insert(rem.clone(), fraction.len());
			rem *= TEN.clone();
			fraction.push_str(&(rem.clone() / self.0.denom()).to_string());
			rem %= self.0.denom();
		}

		let mut output = if self.0.is_negative() {
			"-".to_owned()
		} else {
			String::new()
		};

		output.push_str(&(self.0.numer() / self.0.denom()).to_string());

		if rem.is_zero() {
			if !fraction.is_empty() {
				write!(output, ".{}", &fraction).unwrap();
			}

			Some(output)
		} else {
			None
		}
	}
}

pub enum NativeNumber {
	U64(u64),
	I64(i64),
	F64(f64),
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

/// Error raised when trying to convert a non-decimal number to JSON.
#[derive(Debug, thiserror::Error)]
#[error("not a JSON number: {0}")]
pub struct NonJsonNumber(pub Number);

impl TryFrom<Number> for serde_json::Number {
	type Error = NonJsonNumber;

	fn try_from(value: Number) -> Result<Self, Self::Error> {
		match value.decimal_representation() {
			Some(decimal) => match serde_json::Number::from_str(&decimal) {
				Ok(n) => Ok(n),
				Err(_) => Err(NonJsonNumber(value)),
			},
			None => Err(NonJsonNumber(value)),
		}
	}
}

/// Literal value.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

/// Untyped tree value.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value {
	Literal(Literal),
	Record(BTreeMap<String, Self>),
	List(Vec<Self>),
}

impl Value {
	/// Returns the unit value.
	pub fn unit() -> Self {
		Self::Literal(Literal::Unit)
	}

	/// Checks if the value is unit.
	pub fn is_unit(&self) -> bool {
		matches!(self, Self::Literal(Literal::Unit))
	}
}

impl Default for Value {
	fn default() -> Self {
		Self::unit()
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

/// Error raised when trying to convert a value to JSON that is not compatible
/// with the JSON data model.
#[derive(Debug, thiserror::Error)]
pub enum NonJsonValue {
	/// Number cannot be represented as JSON.
	#[error("not a JSON number: {0}")]
	Number(Number),

	/// Byte string value, not supported by JSON.
	#[error("byte string cannot be converted to JSON")]
	ByteString(Vec<u8>),
}

impl From<NonJsonNumber> for NonJsonValue {
	fn from(value: NonJsonNumber) -> Self {
		NonJsonValue::Number(value.0)
	}
}

impl TryFrom<Literal> for serde_json::Value {
	type Error = NonJsonValue;

	fn try_from(value: Literal) -> Result<Self, Self::Error> {
		match value {
			Literal::Unit => Ok(serde_json::Value::Null),
			Literal::Boolean(b) => Ok(serde_json::Value::Bool(b)),
			Literal::Number(n) => Ok(serde_json::Value::Number(n.try_into()?)),
			Literal::TextString(s) => Ok(serde_json::Value::String(s)),
			Literal::ByteString(s) => Err(NonJsonValue::ByteString(s)),
		}
	}
}

impl TryFrom<TypedLiteral> for serde_json::Value {
	type Error = NonJsonValue;

	fn try_from(value: TypedLiteral) -> Result<Self, Self::Error> {
		match value {
			TypedLiteral::Id(s, _) => Ok(serde_json::Value::String(s)),
			TypedLiteral::Unit(value, _) => value.try_into(),
			TypedLiteral::Boolean(b, _) => Ok(serde_json::Value::Bool(b)),
			TypedLiteral::Number(n, _) => Ok(serde_json::Value::Number(n.try_into()?)),
			TypedLiteral::TextString(s, _) => Ok(serde_json::Value::String(s)),
			TypedLiteral::ByteString(s, _) => Err(NonJsonValue::ByteString(s)),
		}
	}
}

impl TryFrom<Value> for serde_json::Value {
	type Error = NonJsonValue;

	fn try_from(value: Value) -> Result<Self, Self::Error> {
		match value {
			Value::Literal(l) => l.try_into(),
			Value::Record(r) => {
				let mut map = serde_json::Map::new();

				for (key, value) in r {
					map.insert(key, value.try_into()?);
				}

				Ok(serde_json::Value::Object(map))
			}
			Value::List(list) => list
				.into_iter()
				.map(TryInto::try_into)
				.collect::<Result<Vec<_>, _>>()
				.map(serde_json::Value::Array),
		}
	}
}

/// Typed literal value.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypedLiteral<R = rdf_types::Term> {
	/// Unit.
	Unit(Value, Ref<UnitLayoutType, R>),

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
	pub fn type_(&self) -> &Ref<LayoutType, R> {
		match self {
			Self::Unit(_, ty) => ty.as_casted(),
			Self::Boolean(_, ty) => ty.as_casted(),
			Self::Number(_, ty) => ty.as_casted(),
			Self::ByteString(_, ty) => ty.as_casted(),
			Self::TextString(_, ty) => ty.as_casted(),
			Self::Id(_, ty) => ty.as_casted(),
		}
	}

	pub fn into_untyped(self) -> Value {
		match self {
			Self::Unit(value, _) => value,
			Self::Boolean(b, _) => Value::Literal(Literal::Boolean(b)),
			Self::Number(n, _) => Value::Literal(Literal::Number(n)),
			Self::ByteString(s, _) => Value::Literal(Literal::ByteString(s)),
			Self::TextString(s, _) => Value::Literal(Literal::TextString(s)),
			Self::Id(i, _) => Value::Literal(Literal::TextString(i)),
		}
	}
}

/// Typed tree value.
///
/// The "type" information corresponds to the layout used to serialize the
/// value.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypedValue<R = rdf_types::Term> {
	/// Literal value.
	Literal(TypedLiteral<R>),

	/// Variant of a sum layout.
	///
	/// The third parameter is the index of the variant in the sum layout.
	Variant(Box<Self>, Ref<SumLayoutType, R>, u32),

	/// Record.
	Record(BTreeMap<String, Self>, Ref<ProductLayoutType, R>),

	/// List.
	List(Vec<Self>, Ref<ListLayoutType, R>),

	/// Any.
	///
	/// Means that this value has been serialized with the top layout,
	/// accepting any value.
	Always(Value),
}

impl<R> TypedValue<R> {
	/// Returns a reference to the type of this value.
	///
	/// The type is the layout used to serialize/deserialize the value.
	/// In the case of the "top" (also called "any" or "always") layout,
	/// `None` is returned.
	pub fn type_(&self) -> Option<&Ref<LayoutType, R>> {
		match self {
			Self::Literal(t) => Some(t.type_()),
			Self::Variant(_, ty, _) => Some(ty.as_casted()),
			Self::Record(_, ty) => Some(ty.as_casted()),
			Self::List(_, ty) => Some(ty.as_casted()),
			Self::Always(_) => None,
		}
	}

	/// Strips the type information and returns a simple tree value.
	pub fn into_untyped(self) -> Value {
		match self {
			Self::Literal(l) => l.into_untyped(),
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

#[cfg(test)]
mod tests {
	use super::Number;
	use num_rational::Ratio;

	#[test]
	fn decimal_representation() {
		let vectors = [
			((1, 1), Some("1")),
			((1, 2), Some("0.5")),
			((1, 3), None),
			((1, 4), Some("0.25")),
			((1, 5), Some("0.2")),
			((1, 6), None),
			((1, 7), None),
			((1, 8), Some("0.125")),
			((1, 9), None),
		];

		for ((p, q), expected) in vectors {
			let number = Number::new(Ratio::new(p.into(), q.into()));
			assert_eq!(number.decimal_representation().as_deref(), expected)
		}
	}
}
