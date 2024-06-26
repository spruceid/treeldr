use core::fmt;
use std::{collections::BTreeMap, str::FromStr};

use json_syntax::{array::JsonArray, TryFromJson};
use lazy_static::lazy_static;
use num_bigint::{BigInt, Sign};
use num_rational::BigRational;
use num_traits::{Signed, ToPrimitive, Zero};
use xsd_types::ParseXsd;

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
pub mod cbor;

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
			xsd_types::Decimal::parse_xsd(&value.to_string())
				.ok()
				.unwrap()
				.into(),
		)
	}
}

impl From<json_syntax::NumberBuf> for Number {
	fn from(value: json_syntax::NumberBuf) -> Self {
		Self(
			xsd_types::Decimal::parse_xsd(value.as_str())
				.ok()
				.unwrap()
				.into(),
		)
	}
}

impl<'a> From<&'a json_syntax::Number> for Number {
	fn from(value: &'a json_syntax::Number) -> Self {
		Self(
			xsd_types::Decimal::parse_xsd(value.as_str())
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

impl TryFrom<Number> for json_syntax::NumberBuf {
	type Error = NonJsonNumber;

	fn try_from(value: Number) -> Result<Self, Self::Error> {
		match value.decimal_representation() {
			Some(decimal) => match json_syntax::NumberBuf::from_str(&decimal) {
				Ok(n) => Ok(n),
				Err(_) => Err(NonJsonNumber(value)),
			},
			None => Err(NonJsonNumber(value)),
		}
	}
}

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

impl Literal {
	pub fn as_str(&self) -> Option<&str> {
		match self {
			Self::TextString(s) => Some(s),
			_ => None,
		}
	}
}

impl fmt::Display for Literal {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Unit => f.write_str("null"),
			Self::Boolean(true) => f.write_str("true"),
			Self::Boolean(false) => f.write_str("false"),
			Self::Number(n) => n.fmt(f),
			Self::ByteString(bytes) => {
				f.write_str("x")?;
				for b in bytes {
					write!(f, "{b:02x}")?;
				}
				Ok(())
			}
			Self::TextString(s) => json_syntax::print::string_literal(s, f),
		}
	}
}

/// Untyped tree value.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value {
	Literal(Literal),
	Map(BTreeMap<Self, Self>),
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

	/// Creates a text string value.
	pub fn string(value: String) -> Self {
		Self::Literal(Literal::TextString(value))
	}

	pub fn as_str(&self) -> Option<&str> {
		match self {
			Self::Literal(l) => l.as_str(),
			_ => None,
		}
	}
}

impl Default for Value {
	fn default() -> Self {
		Self::unit()
	}
}

impl TryFromJson for Value {
	type Error = std::convert::Infallible;

	fn try_from_json_at(
		json: &json_syntax::Value,
		code_map: &json_syntax::CodeMap,
		offset: usize,
	) -> Result<Self, Self::Error> {
		match json {
			json_syntax::Value::Null => Ok(Self::Literal(Literal::Unit)),
			json_syntax::Value::Boolean(b) => Ok(Self::Literal(Literal::Boolean(*b))),
			json_syntax::Value::Number(n) => {
				Ok(Self::Literal(Literal::Number(n.as_number().into())))
			}
			json_syntax::Value::String(s) => Ok(Self::Literal(Literal::TextString(s.to_string()))),
			json_syntax::Value::Array(a) => Ok(Self::List(
				a.iter_mapped(code_map, offset)
					.map(|item| Self::try_from_json_at(item.value, code_map, item.offset).unwrap())
					.collect(),
			)),
			json_syntax::Value::Object(o) => Ok(Self::Map(
				o.iter_mapped(code_map, offset)
					.map(|entry| {
						(
							Value::string(entry.value.key.value.to_string()),
							Self::try_from_json_at(
								entry.value.value.value,
								code_map,
								entry.value.value.offset,
							)
							.unwrap(),
						)
					})
					.collect(),
			)),
		}
	}
}

impl From<json_syntax::Value> for Value {
	fn from(value: json_syntax::Value) -> Self {
		match value {
			json_syntax::Value::Null => Self::Literal(Literal::Unit),
			json_syntax::Value::Boolean(b) => Self::Literal(Literal::Boolean(b)),
			json_syntax::Value::Number(n) => Self::Literal(Literal::Number(n.into())),
			json_syntax::Value::String(s) => Self::Literal(Literal::TextString(s.to_string())),
			json_syntax::Value::Array(a) => Self::List(a.into_iter().map(Into::into).collect()),
			json_syntax::Value::Object(o) => Self::Map(
				o.into_iter()
					.map(|entry| (Value::string(entry.key.into_string()), entry.value.into()))
					.collect(),
			),
		}
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
			serde_json::Value::Object(entries) => Self::Map(
				entries
					.into_iter()
					.map(|(key, value)| (Value::string(key), value.into()))
					.collect(),
			),
		}
	}
}

impl fmt::Display for Value {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Literal(l) => l.fmt(f),
			Self::List(l) => {
				f.write_str("[")?;
				for (i, v) in l.iter().enumerate() {
					if i > 0 {
						f.write_str(",")?;
					}

					v.fmt(f)?;
				}
				f.write_str("]")
			}
			Self::Map(m) => {
				f.write_str("{")?;
				for (i, (k, v)) in m.iter().enumerate() {
					if i > 0 {
						f.write_str(",")?;
					}

					k.fmt(f)?;
					f.write_str(":")?;
					v.fmt(f)?;
				}
				f.write_str("}")
			}
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

	/// Non-string key, not supported by JSON.
	#[error("non-string key")]
	NonStringKey(Value),
}

impl From<NonJsonNumber> for NonJsonValue {
	fn from(value: NonJsonNumber) -> Self {
		NonJsonValue::Number(value.0)
	}
}

impl TryFrom<Literal> for json_syntax::Value {
	type Error = NonJsonValue;

	fn try_from(value: Literal) -> Result<Self, Self::Error> {
		match value {
			Literal::Unit => Ok(json_syntax::Value::Null),
			Literal::Boolean(b) => Ok(json_syntax::Value::Boolean(b)),
			Literal::Number(n) => Ok(json_syntax::Value::Number(n.try_into()?)),
			Literal::TextString(s) => Ok(json_syntax::Value::String(s.into())),
			Literal::ByteString(s) => Err(NonJsonValue::ByteString(s)),
		}
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

impl TryFrom<Value> for json_syntax::Value {
	type Error = NonJsonValue;

	fn try_from(value: Value) -> Result<Self, Self::Error> {
		match value {
			Value::Literal(l) => l.try_into(),
			Value::Map(r) => {
				let mut object = json_syntax::Object::new();

				for (key, value) in r {
					match key {
						Value::Literal(Literal::TextString(key)) => {
							object.insert(key.into(), value.try_into()?);
						}
						other => return Err(NonJsonValue::NonStringKey(other)),
					}
				}

				Ok(json_syntax::Value::Object(object))
			}
			Value::List(list) => list
				.into_iter()
				.map(TryInto::try_into)
				.collect::<Result<Vec<_>, _>>()
				.map(json_syntax::Value::Array),
		}
	}
}

impl TryFrom<Value> for serde_json::Value {
	type Error = NonJsonValue;

	fn try_from(value: Value) -> Result<Self, Self::Error> {
		match value {
			Value::Literal(l) => l.try_into(),
			Value::Map(r) => {
				let mut object = serde_json::Map::new();

				for (key, value) in r {
					match key {
						Value::Literal(Literal::TextString(key)) => {
							object.insert(key, value.try_into()?);
						}
						other => return Err(NonJsonValue::NonStringKey(other)),
					}
				}

				Ok(serde_json::Value::Object(object))
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

	/// Map.
	Map(BTreeMap<Value, Self>, Ref<ProductLayoutType, R>),

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
			Self::Map(_, ty) => Some(ty.as_casted()),
			Self::List(_, ty) => Some(ty.as_casted()),
			Self::Always(_) => None,
		}
	}

	/// Strips the type information and returns a simple tree value.
	pub fn into_untyped(self) -> Value {
		match self {
			Self::Literal(l) => l.into_untyped(),
			Self::Variant(value, _, _) => value.into_untyped(),
			Self::Map(map, _) => Value::Map(
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
