//! Common functions for code generation.
use std::fmt;

pub mod doc;

/// Indentation method.
#[derive(Clone, Copy)]
pub enum Indent {
	/// Use a tab `\t` character.
	Tab,

	/// Use the given number of spaces.
	Spaces(u8),
}

impl Indent {
	pub fn by(&self, n: u32) -> IndentBy {
		IndentBy(*self, n)
	}
}

impl fmt::Display for Indent {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Tab => write!(f, "\t"),
			Self::Spaces(n) => {
				for _ in 0..*n {
					write!(f, " ")?;
				}

				Ok(())
			}
		}
	}
}

#[derive(Clone, Copy)]
pub struct IndentBy(Indent, u32);

impl fmt::Display for IndentBy {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for _ in 0..self.1 {
			self.0.fmt(f)?;
		}

		Ok(())
	}
}
