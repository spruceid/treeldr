use super::{Decimal, Integer};
use crate::vocab::StrippedLiteral;
use num::{BigInt, BigRational, Signed, Zero};
use std::fmt;
use std::fmt::Write;

lazy_static::lazy_static! {
	static ref TEN: BigInt = 10u32.into();
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Rational(BigRational);

impl Rational {
	pub fn numer(&self) -> &Integer {
		unsafe { core::mem::transmute(self.0.numer()) }
	}

	pub fn denom(&self) -> &Integer {
		unsafe { core::mem::transmute(self.0.denom()) }
	}

	pub fn is_zero(&self) -> bool {
		self.0.is_zero()
	}

	pub fn is_integer(&self) -> bool {
		self.0.is_integer()
	}

	pub fn as_integer(&self) -> Option<&Integer> {
		if self.is_integer() {
			Some(self.numer())
		} else {
			None
		}
	}

	pub fn is_negative(&self) -> bool {
		self.0.is_negative()
	}

	pub fn is_decimal(&self) -> bool {
		let mut set = std::collections::HashSet::new();

		let mut rem = if self.is_negative() {
			-self.0.numer()
		} else {
			self.0.numer().clone()
		};

		rem %= self.0.denom();
		while !rem.is_zero() && !set.contains(&rem) {
			set.insert(rem.clone());
			rem = (rem * TEN.clone()) % self.0.denom();
		}

		rem.is_zero()
	}

	pub fn as_lexical(&self) -> Option<&Decimal> {
		if self.is_decimal() {
			Some(unsafe { core::mem::transmute(self) })
		} else {
			None
		}
	}

	pub fn lexical_decimal(&self) -> Option<xsd_types::DecimalBuf> {
		let mut fraction = String::new();
		let mut map = std::collections::HashMap::new();

		let mut rem = if self.is_negative() {
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

		let mut output = if self.is_negative() {
			"-".to_owned()
		} else {
			String::new()
		};

		output.push_str(&(self.0.numer() / self.0.denom()).to_string());

		if rem.is_zero() {
			if !fraction.is_empty() {
				write!(output, ".{}", &fraction).unwrap();
			}

			Some(unsafe { xsd_types::DecimalBuf::new_unchecked(output) })
		} else {
			None
		}
	}

	pub fn literal(&self) -> StrippedLiteral {
		use crate::vocab::{Owl, Term, Xsd};
		match self.lexical_decimal() {
			Some(decimal) => {
				StrippedLiteral::TypedString(decimal.into_string().into(), Term::Xsd(Xsd::Decimal))
			}
			None => {
				StrippedLiteral::TypedString(self.0.to_string().into(), Term::Owl(Owl::Rational))
			}
		}
	}
}

impl fmt::Display for Rational {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(f)
	}
}
