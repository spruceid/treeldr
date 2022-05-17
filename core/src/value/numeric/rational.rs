use std::fmt::Write;
use num::{
	BigInt,
	BigRational,
	Zero,
	Signed
};

lazy_static::lazy_static! {
	static ref TEN: BigInt = 10u32.into();
}

pub type Rational = BigRational;

pub trait Extra {
	/// Checks if this rational number is also a decimal number.
	fn is_decimal(&self) -> bool;

	/// Converts this rational number into a decimal number if possible.
	fn to_decimal(&self) -> Option<xsd_types::DecimalBuf>;
}

impl Extra for Rational {
	fn is_decimal(&self) -> bool {
		let mut set = std::collections::HashSet::new();

		let mut rem = if self.is_negative() {
			-self.numer()
		} else {
			self.numer().clone()
		};

		rem = rem % self.denom();
		while !rem.is_zero() && !set.contains(&rem) {
			set.insert(rem.clone());
			rem = (rem * TEN.clone()) % self.denom();
		}

		rem.is_zero()
	}

	fn to_decimal(&self) -> Option<xsd_types::DecimalBuf> {
		let mut fraction = String::new();
		let mut map = std::collections::HashMap::new();

		let mut rem = if self.is_negative() {
			-self.numer()
		} else {
			self.numer().clone()
		};

		rem = rem % self.denom();
		while !rem.is_zero() && !map.contains_key(&rem) {
			map.insert(rem.clone(), fraction.len());
			rem *= TEN.clone();
			fraction.push_str(&(rem.clone() / self.denom()).to_string());
			rem = rem % self.denom();
		}

		let mut output = if self.is_negative() {
			"-".to_owned()
		} else {
			String::new()
		};
		
		output.push_str(&(self.numer() / self.denom()).to_string());
		
		if rem.is_zero() {
			if fraction.len() != 0 {
				write!(output, ".{}", &fraction).unwrap();
			}

			Some(unsafe {
				xsd_types::DecimalBuf::new_unchecked(output)
			})
		} else {
			None
		}
	}
}
