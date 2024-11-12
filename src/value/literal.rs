use core::fmt;
use num_bigint::{BigInt, Sign};
use num_rational::BigRational;
use num_traits::{Signed, ToPrimitive, Zero};
use std::{str::FromStr, sync::LazyLock};

pub use xsd_types::NoDecimalRepresentation;

static TEN: LazyLock<BigInt> = LazyLock::new(|| 10u32.into());

pub static NUMBER_ZERO: LazyLock<Number> = LazyLock::new(|| 0u32.into());
pub static NUMBER_U8_MAX: LazyLock<Number> = LazyLock::new(|| u8::MAX.into());
pub static NUMBER_U16_MAX: LazyLock<Number> = LazyLock::new(|| u16::MAX.into());
pub static NUMBER_U32_MAX: LazyLock<Number> = LazyLock::new(|| u32::MAX.into());
pub static NUMBER_U64_MAX: LazyLock<Number> = LazyLock::new(|| u64::MAX.into());
pub static NUMBER_U128_MAX: LazyLock<Number> = LazyLock::new(|| u64::MAX.into());
pub static NUMBER_I8_MIN: LazyLock<Number> = LazyLock::new(|| i8::MIN.into());
pub static NUMBER_I16_MIN: LazyLock<Number> = LazyLock::new(|| i16::MIN.into());
pub static NUMBER_I32_MIN: LazyLock<Number> = LazyLock::new(|| i32::MIN.into());
pub static NUMBER_I64_MIN: LazyLock<Number> = LazyLock::new(|| i64::MIN.into());
pub static NUMBER_I128_MIN: LazyLock<Number> = LazyLock::new(|| i64::MIN.into());
pub static NUMBER_I8_MAX: LazyLock<Number> = LazyLock::new(|| i8::MAX.into());
pub static NUMBER_I16_MAX: LazyLock<Number> = LazyLock::new(|| i16::MAX.into());
pub static NUMBER_I32_MAX: LazyLock<Number> = LazyLock::new(|| i32::MAX.into());
pub static NUMBER_I64_MAX: LazyLock<Number> = LazyLock::new(|| i64::MAX.into());
pub static NUMBER_I128_MAX: LazyLock<Number> = LazyLock::new(|| i64::MAX.into());

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
			Some(c) => match c.to_digit(10) {
				Some(d) => {
					numer *= 10;
					numer += d;
					empty = false;
				}
				None => return Err(NumberParseError::InvalidDigit),
			},
			None => (),
		}

		while let Some(c) = chars.next() {
			match c {
				'.' if !fractional_part => {
					if empty {
						return Err(NumberParseError::MissingNumer);
					}

					fractional_part = true;
					empty = true;
					denom *= 10;
				}
				_ => match c.to_digit(10) {
					Some(d) => {
						numer *= 10;
						numer += d;

						if fractional_part {
							denom *= 10;
						}

						empty = false;
					}
					None => return Err(NumberParseError::InvalidDigit),
				},
			}
		}

		if empty {
			if fractional_part {
				return Err(NumberParseError::MissingDenom);
			} else {
				return Err(NumberParseError::MissingNumer);
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
	InvalidDigit,
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
			None => Number::from_decimal(s),
		}
	}
}

impl From<xsd_types::Decimal> for Number {
	fn from(value: xsd_types::Decimal) -> Self {
		Self(value.into())
	}
}

impl TryFrom<Number> for xsd_types::Decimal {
	type Error = NoDecimalRepresentation;

	fn try_from(value: Number) -> Result<Self, Self::Error> {
		value.0.try_into()
	}
}

impl From<xsd_types::Integer> for Number {
	fn from(value: xsd_types::Integer) -> Self {
		let n: BigInt = value.into();
		Self(n.into())
	}
}

#[derive(Debug, thiserror::Error)]
#[error("not an integer")]
pub struct NotAnInteger;

impl TryFrom<Number> for xsd_types::Integer {
	type Error = NotAnInteger;

	fn try_from(value: Number) -> Result<Self, Self::Error> {
		if value.0.is_integer() {
			Ok(value.0.into_raw().0.into())
		} else {
			Err(NotAnInteger)
		}
	}
}

impl From<xsd_types::NonNegativeInteger> for Number {
	fn from(value: xsd_types::NonNegativeInteger) -> Self {
		let n: BigInt = value.into();
		Self(n.into())
	}
}

#[derive(Debug, thiserror::Error)]
#[error("not a non-negative integer")]
pub struct NotANonNegativeInteger;

impl TryFrom<Number> for xsd_types::NonNegativeInteger {
	type Error = NotANonNegativeInteger;

	fn try_from(value: Number) -> Result<Self, Self::Error> {
		if value.0.is_integer() {
			let numer = value.0.into_raw().0;
			if numer.is_negative() {
				Err(NotANonNegativeInteger)
			} else {
				Ok(unsafe { Self::new_unchecked(numer) })
			}
		} else {
			Err(NotANonNegativeInteger)
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
	pub fn as_bool(&self) -> Option<bool> {
		match self {
			Self::Boolean(b) => Some(*b),
			_ => None,
		}
	}

	pub fn as_number(&self) -> Option<&Number> {
		match self {
			Self::Number(n) => Some(n),
			_ => None,
		}
	}

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
			Self::TextString(s) => string_literal(s, f),
		}
	}
}

/// Formats a string literal according to [RFC8785](https://www.rfc-editor.org/rfc/rfc8785#name-serialization-of-strings).
pub fn string_literal(s: &str, f: &mut fmt::Formatter) -> fmt::Result {
	use fmt::Display;
	f.write_str("\"")?;

	for c in s.chars() {
		match c {
			'\\' => f.write_str("\\\\")?,
			'\"' => f.write_str("\\\"")?,
			'\u{0008}' => f.write_str("\\b")?,
			'\u{0009}' => f.write_str("\\t")?,
			'\u{000a}' => f.write_str("\\n")?,
			'\u{000c}' => f.write_str("\\f")?,
			'\u{000d}' => f.write_str("\\r")?,
			'\u{0000}'..='\u{001f}' => {
				f.write_str("\\u")?;

				let codepoint = c as u32;
				let d = codepoint & 0x000f;
				let c = (codepoint & 0x00f0) >> 4;
				let b = (codepoint & 0x0f00) >> 8;
				let a = (codepoint & 0xf000) >> 12;

				digit(a).fmt(f)?;
				digit(b).fmt(f)?;
				digit(c).fmt(f)?;
				digit(d).fmt(f)?
			}
			_ => c.fmt(f)?,
		}
	}

	f.write_str("\"")
}

fn digit(c: u32) -> char {
	match c {
		0x0 => '0',
		0x1 => '1',
		0x2 => '2',
		0x3 => '3',
		0x4 => '4',
		0x5 => '5',
		0x6 => '6',
		0x7 => '7',
		0x8 => '8',
		0x9 => '9',
		0xa => 'a',
		0xb => 'b',
		0xc => 'c',
		0xd => 'd',
		0xe => 'e',
		0xf => 'f',
		_ => panic!("invalid input: {}", c),
	}
}
