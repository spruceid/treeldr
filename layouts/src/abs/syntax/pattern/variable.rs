use core::fmt;
use std::borrow::Borrow;

#[derive(Debug, thiserror::Error)]
#[error("invalid variable name `{0}`")]
pub struct InvalidVariableName<T = String>(pub T);

/// Variable name.
///
/// Subset of `str` that can serve as a variable name in the abstract syntax.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct VariableName(str);

impl VariableName {
	pub const SELF: &'static Self = unsafe { Self::new_unchecked("self") };

	pub const VALUE: &'static Self = unsafe { Self::new_unchecked("value") };

	/// Parses the given string and turns it into a variable name.
	pub fn new(s: &str) -> Result<&Self, InvalidVariableName<&str>> {
		if check_variable_name(s.chars()) {
			Ok(unsafe { Self::new_unchecked(s) })
		} else {
			Err(InvalidVariableName(s))
		}
	}

	/// Converts the given string into a variable name without parsing.
	///
	/// # Safety
	///
	/// The input string **must** be a valid variable name.
	pub const unsafe fn new_unchecked(s: &str) -> &Self {
		std::mem::transmute(s)
	}

	pub fn as_str(&self) -> &str {
		&self.0
	}
}

impl ToOwned for VariableName {
	type Owned = VariableNameBuf;

	fn to_owned(&self) -> Self::Owned {
		VariableNameBuf(self.0.to_owned())
	}
}

impl PartialEq<str> for VariableName {
	fn eq(&self, other: &str) -> bool {
		&self.0 == other
	}
}

impl std::ops::Deref for VariableName {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		self.as_str()
	}
}

impl Borrow<str> for VariableName {
	fn borrow(&self) -> &str {
		self.as_str()
	}
}

impl AsRef<str> for VariableName {
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

fn check_variable_name<C: Iterator<Item = char>>(mut chars: C) -> bool {
	match chars.next() {
		Some(c) if c.is_ascii_digit() || is_pn_char_u(c) => {
			for c in chars {
				if !is_pn_char(c) {
					return false;
				}
			}

			true
		}
		_ => false,
	}
}

fn is_pn_char_base(c: char) -> bool {
	matches!(c, 'A'..='Z' | 'a'..='z' | '\u{00c0}'..='\u{00d6}' | '\u{00d8}'..='\u{00f6}' | '\u{00f8}'..='\u{02ff}' | '\u{0370}'..='\u{037d}' | '\u{037f}'..='\u{1fff}' | '\u{200c}'..='\u{200d}' | '\u{2070}'..='\u{218f}' | '\u{2c00}'..='\u{2fef}' | '\u{3001}'..='\u{d7ff}' | '\u{f900}'..='\u{fdcf}' | '\u{fdf0}'..='\u{fffd}' | '\u{10000}'..='\u{effff}')
}

fn is_pn_char_u(c: char) -> bool {
	is_pn_char_base(c) || matches!(c, '_' | ':')
}

fn is_pn_char(c: char) -> bool {
	is_pn_char_u(c)
		|| matches!(c, '-' | '0'..='9' | '\u{00b7}' | '\u{0300}'..='\u{036f}' | '\u{203f}'..='\u{2040}')
}

/// Variable name buffer.
///
/// Subset of [`String`] that can serve as a variable name in the abstract syntax.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VariableNameBuf(pub(crate) String);

impl VariableNameBuf {
	/// Parses the given string to create a new variable name.
	pub fn new(s: String) -> Result<Self, InvalidVariableName> {
		if check_variable_name(s.chars()) {
			Ok(Self(s))
		} else {
			Err(InvalidVariableName(s))
		}
	}

	pub fn default_head() -> Self {
		Self("self".to_string())
	}

	/// Converts the given string into a variable name without parsing.
	///
	/// # Safety
	///
	/// The input string **must** be a valid variable name.
	pub unsafe fn new_unchecked(s: String) -> Self {
		Self(s)
	}

	pub fn as_variable_name(&self) -> &VariableName {
		unsafe { VariableName::new_unchecked(&self.0) }
	}

	pub fn into_string(self) -> String {
		self.0
	}
}

impl std::ops::Deref for VariableNameBuf {
	type Target = VariableName;

	fn deref(&self) -> &Self::Target {
		self.as_variable_name()
	}
}

impl Borrow<VariableName> for VariableNameBuf {
	fn borrow(&self) -> &VariableName {
		self.as_variable_name()
	}
}

impl AsRef<VariableName> for VariableNameBuf {
	fn as_ref(&self) -> &VariableName {
		self.as_variable_name()
	}
}

impl PartialEq<str> for VariableNameBuf {
	fn eq(&self, other: &str) -> bool {
		self.0 == other
	}
}

impl fmt::Display for VariableNameBuf {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.0.fmt(f)
	}
}
