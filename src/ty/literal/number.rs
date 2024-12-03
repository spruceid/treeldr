use core::fmt;
use std::cmp::Ordering;

use serde_seeded::{DeserializeSeeded, SerializeSeeded};

use crate::{
	display::DisplaySignature,
	value::{
		self,
		number::{
			NUMBER_I128_MAX, NUMBER_I128_MIN, NUMBER_I16_MAX, NUMBER_I16_MIN, NUMBER_I32_MAX,
			NUMBER_I32_MIN, NUMBER_I64_MAX, NUMBER_I64_MIN, NUMBER_I8_MAX, NUMBER_I8_MIN,
			NUMBER_U128_MAX, NUMBER_U16_MAX, NUMBER_U32_MAX, NUMBER_U64_MAX, NUMBER_U8_MAX,
			NUMBER_ZERO,
		},
	},
};

#[derive(
	Debug,
	Default,
	Clone,
	Copy,
	PartialEq,
	Eq,
	PartialOrd,
	Ord,
	Hash,
	SerializeSeeded,
	DeserializeSeeded,
)]
#[seeded(ser(params(Q), seed(Q)), de(params(Q), seed(Q)))]
pub enum NumberDomain {
	/// Rational numbers (ℚ).
	#[default]
	Rational,

	/// Integer numbers (ℤ).
	Integer,
}

impl NumberDomain {
	pub fn is_integer(&self) -> bool {
		matches!(self, Self::Integer)
	}
}

#[derive(
	Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SerializeSeeded, DeserializeSeeded,
)]
#[seeded(ser(params(Q), seed(Q)), de(params(Q), seed(Q)))]
pub enum MinBound {
	#[default]
	Unbounded,
	Excluded(value::Number),
	Included(value::Number),
}

impl MinBound {
	pub fn is_unbounded(&self) -> bool {
		matches!(self, Self::Unbounded)
	}

	pub fn normalize(&mut self, domain: NumberDomain) {
		if domain.is_integer() {
			if let Self::Excluded(n) = &self {
				*self = Self::Included(n.clone() + 1)
			}
		}
	}
}

#[derive(
	Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SerializeSeeded, DeserializeSeeded,
)]
#[seeded(ser(params(Q), seed(Q)), de(params(Q), seed(Q)))]
pub enum MaxBound {
	Included(value::Number),
	Excluded(value::Number),
	#[default]
	Unbounded,
}

impl MaxBound {
	pub fn is_unbounded(&self) -> bool {
		matches!(self, Self::Unbounded)
	}

	pub fn as_included(&self) -> Option<&value::Number> {
		match self {
			Self::Included(m) => Some(m),
			_ => None,
		}
	}

	pub fn normalize(&mut self, domain: NumberDomain) {
		if domain.is_integer() {
			if let Self::Excluded(n) = &self {
				*self = Self::Included(n.clone() - 1)
			}
		}
	}
}

#[derive(
	Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SerializeSeeded, DeserializeSeeded,
)]
#[seeded(ser(params(Q), seed(Q)), de(params(Q), seed(Q)))]
pub struct Number {
	/// Number domain.
	///
	/// NOTE: this is not public because the `min` and `max` bounds are
	/// normalized differently depending on this value.
	domain: NumberDomain,

	/// Min bound.
	///
	/// NOTE: this is not public because it is normalized differently depending
	/// on the `domain` value.
	#[seeded(default, skip_serializing_if(MinBound::is_unbounded))]
	min: MinBound,

	/// Max bound.
	///
	/// NOTE: this is not public because it is normalized differently depending
	/// on the `domain` value.
	#[seeded(default, skip_serializing_if(MaxBound::is_unbounded))]
	max: MaxBound,
}

impl Number {
	pub fn singleton(value: value::Number) -> Self {
		let domain = if value.is_integer() {
			NumberDomain::Integer
		} else {
			NumberDomain::Rational
		};

		Self {
			domain,
			min: MinBound::Included(value.clone()),
			max: MaxBound::Included(value),
		}
	}

	pub fn int() -> Self {
		Self {
			domain: NumberDomain::Integer,
			min: MinBound::Unbounded,
			max: MaxBound::Unbounded,
		}
	}

	pub fn uint() -> Self {
		Self {
			domain: NumberDomain::Integer,
			min: MinBound::Included(NUMBER_ZERO.clone()),
			max: MaxBound::Unbounded,
		}
	}

	pub fn as_singleton(&self) -> Option<&value::Number> {
		match (&self.min, &self.max) {
			(MinBound::Included(a), MaxBound::Included(b)) if a == b => Some(a),
			_ => None,
		}
	}

	pub fn subtype_cmp(&self, other: &Self) -> Option<Ordering> {
		match self.min.cmp(&other.min) {
			Ordering::Less => match self.max.cmp(&other.max) {
				Ordering::Less => None,
				Ordering::Greater | Ordering::Equal => match (self.domain, other.domain) {
					(NumberDomain::Rational, _)
					| (NumberDomain::Integer, NumberDomain::Integer) => Some(Ordering::Greater),
					_ => None,
				},
			},
			Ordering::Equal => match self.max.cmp(&other.max) {
				Ordering::Less => match (self.domain, other.domain) {
					(_, NumberDomain::Rational)
					| (NumberDomain::Integer, NumberDomain::Integer) => Some(Ordering::Less),
					_ => None,
				},
				Ordering::Equal => match (self.domain, other.domain) {
					(NumberDomain::Rational, NumberDomain::Rational)
					| (NumberDomain::Integer, NumberDomain::Integer) => Some(Ordering::Equal),
					(NumberDomain::Rational, NumberDomain::Integer) => Some(Ordering::Greater),
					_ => None,
				},
				Ordering::Greater => match (self.domain, other.domain) {
					(NumberDomain::Rational, _)
					| (NumberDomain::Integer, NumberDomain::Integer) => Some(Ordering::Greater),
					_ => None,
				},
			},
			Ordering::Greater => match self.max.cmp(&other.max) {
				Ordering::Less | Ordering::Equal => match (self.domain, other.domain) {
					(_, NumberDomain::Rational)
					| (NumberDomain::Integer, NumberDomain::Integer) => Some(Ordering::Less),
					_ => None,
				},
				Ordering::Greater => None,
			},
		}
	}

	pub fn contains(&self, _value: &value::Number) -> bool {
		true
	}

	pub fn domain(&self) -> NumberDomain {
		self.domain
	}

	pub fn set_domain(&mut self, d: NumberDomain) {
		self.domain = d;
		self.min.normalize(d);
		self.max.normalize(d);
	}

	pub fn min(&self) -> &MinBound {
		&self.min
	}

	pub fn set_min(&mut self, bound: MinBound) {
		self.min = bound;
		self.min.normalize(self.domain);
	}

	pub fn max(&self) -> &MaxBound {
		&self.max
	}

	pub fn set_max(&mut self, bound: MaxBound) {
		self.max = bound;
		self.max.normalize(self.domain);
	}

	fn as_native_number(&self) -> Option<NativeNumber> {
		match self.domain {
			NumberDomain::Integer => {
				// int
				match &self.min {
					MinBound::Unbounded => Some(NativeNumber::Int),
					MinBound::Included(n) => {
						if *n == *NUMBER_ZERO {
							match &self.max {
								MaxBound::Unbounded => return Some(NativeNumber::Uint),
								MaxBound::Included(m) => {
									if *m == *NUMBER_U8_MAX {
										return Some(NativeNumber::U8);
									}

									if *m == *NUMBER_U16_MAX {
										return Some(NativeNumber::U16);
									}

									if *m == *NUMBER_U32_MAX {
										return Some(NativeNumber::U32);
									}

									if *m == *NUMBER_U64_MAX {
										return Some(NativeNumber::U64);
									}

									if *m == *NUMBER_U128_MAX {
										return Some(NativeNumber::U128);
									}
								}
								MaxBound::Excluded(_) => panic!("non normalized bound"),
							}
						}

						if *n == *NUMBER_I8_MIN
							&& self.max.as_included().is_some_and(|m| *m == *NUMBER_I8_MAX)
						{
							return Some(NativeNumber::I8);
						}

						if *n == *NUMBER_I16_MIN
							&& self
								.max
								.as_included()
								.is_some_and(|m| *m == *NUMBER_I16_MAX)
						{
							return Some(NativeNumber::I16);
						}

						if *n == *NUMBER_I32_MIN
							&& self
								.max
								.as_included()
								.is_some_and(|m| *m == *NUMBER_I32_MAX)
						{
							return Some(NativeNumber::I32);
						}

						if *n == *NUMBER_I64_MIN
							&& self
								.max
								.as_included()
								.is_some_and(|m| *m == *NUMBER_I64_MAX)
						{
							return Some(NativeNumber::I64);
						}

						if *n == *NUMBER_I128_MIN
							&& self
								.max
								.as_included()
								.is_some_and(|m| *m == *NUMBER_I128_MAX)
						{
							return Some(NativeNumber::I128);
						}

						None
					}
					MinBound::Excluded(_) => panic!("non normalized bound"),
				}
			}
			NumberDomain::Rational => {
				if self.min.is_unbounded() && self.max.is_unbounded() {
					Some(NativeNumber::Rational)
				} else {
					None
				}
			}
		}
	}
}

impl<P, R> DisplaySignature<P, R> for Number {
	fn fmt_signature(
		&self,
		_context: &crate::display::DisplayContext<
			impl crate::Registry<PackageId = P, Resource = R>,
		>,
		f: &mut fmt::Formatter,
	) -> fmt::Result {
		match self.as_singleton() {
			Some(s) => fmt::Display::fmt(s, f),
			None => match self.as_native_number() {
				Some(n) => match n {
					NativeNumber::Uint => write!(f, "uint"),
					NativeNumber::U8 => write!(f, "u8"),
					NativeNumber::U16 => write!(f, "u16"),
					NativeNumber::U32 => write!(f, "u32"),
					NativeNumber::U64 => write!(f, "u64"),
					NativeNumber::U128 => write!(f, "u128"),
					NativeNumber::Int => write!(f, "int"),
					NativeNumber::I8 => write!(f, "i8"),
					NativeNumber::I16 => write!(f, "i16"),
					NativeNumber::I32 => write!(f, "i32"),
					NativeNumber::I64 => write!(f, "i64"),
					NativeNumber::I128 => write!(f, "i128"),
					NativeNumber::Rational => write!(f, "rational"),
				},
				None => {
					todo!()
				}
			},
		}
	}
}

pub enum NativeNumber {
	Uint,
	U8,
	U16,
	U32,
	U64,
	U128,
	Int,
	I8,
	I16,
	I32,
	I64,
	I128,
	Rational,
}
