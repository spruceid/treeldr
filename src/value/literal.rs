use core::fmt;
use num_bigint::{BigInt, Sign};
use num_rational::BigRational;
use num_traits::{Signed, ToPrimitive, Zero};
use std::{str::FromStr, sync::LazyLock};
use xsd_types::ParseXsd;

static TEN: LazyLock<BigInt> = LazyLock::new(|| 10u32.into());

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

	pub fn from_decimal(s: &str) -> Result<Self, NumberParseError> {
		let mut numer: BigInt = 0.into();
		let mut denom: BigInt = 1.into();
		let mut negate = false;
		let mut fractional_part = false;
		let mut empty = true;

		let mut chars = s.chars();

		match chars.next() {
			Some('+') => (),
			Some('-') => {
				negate = true;
			}
			Some(c) => {
				match c.to_digit(10) {
					Some(d) => {
						numer *= 10;
						numer += d;
						empty = false;
					}
					None => {
						return Err(NumberParseError::InvalidDigit)
					}
				}
			}
			None => ()
		}

		while let Some(c) = chars.next() {
			match c {
				'.' if !fractional_part => {
					if empty {
						return Err(NumberParseError::MissingNumer)
					}

					fractional_part = true;
					empty = true;
					denom *= 10;
				},
				_ => {
					match c.to_digit(10) {
						Some(d) => {
							numer *= 10;
							numer += d;

							if fractional_part {
								denom *= 10;
							}

							empty = false;
						}
						None => {
							return Err(NumberParseError::InvalidDigit)
						}
					}
				}
			}
		}

		if empty {
			if fractional_part {
				return Err(NumberParseError::MissingDenom)
			} else {
				return Err(NumberParseError::MissingNumer)
			}
		}

		if negate {
			numer = -numer;
		}

		Ok(Number(BigRational::new(numer, denom)))
	}
}

#[derive(Debug, thiserror::Error)]
pub enum NumberParseError {
	#[error("missing numerator")]
	MissingNumer,

	#[error("missing denominator")]
	MissingDenom,

	#[error("denominator is zero")]
	ZeroDenominator,

	#[error("invalid digit")]
	InvalidDigit
}

impl FromStr for Number {
	type Err = NumberParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.split_once('/') {
			Some((numer, denom)) => {
				let numer = Number::from_decimal(numer)?.0;
				let denom = Number::from_decimal(denom)?.0;

				if denom.is_zero() {
					Err(NumberParseError::ZeroDenominator)
				} else {
					Ok(Number(numer / denom))
				}
			}
			None => {
				Number::from_decimal(s)
			}
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

			impl std::ops::Add<$ty> for Number {
				type Output = Self;

				fn add(self, other: $ty) -> Self::Output {
					Self(self.0 + Self::from(other).0)
				}
			}

			impl std::ops::Sub<$ty> for Number {
				type Output = Self;

				fn sub(self, other: $ty) -> Self::Output {
					Self(self.0 + Self::from(other).0)
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
			Some(decimal) => match json_syntax::NumberBuf::new(
				json_syntax::number::Buffer::from_vec(decimal.into_bytes()),
			) {
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
