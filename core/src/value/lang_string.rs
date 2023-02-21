use langtag::{LanguageTag, LanguageTagBuf};
use rdf_types::RdfDisplay;
use std::{borrow::Borrow, fmt, ops::Deref};

/// Language tagged string.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LangString {
	value: String,
	language: LanguageTagBuf,
}

impl LangString {
	pub fn new(value: String, language: LanguageTagBuf) -> Self {
		Self { value, language }
	}

	pub fn as_str(&self) -> &str {
		&self.value
	}

	pub fn language(&self) -> LanguageTag {
		self.language.as_ref()
	}
}

impl fmt::Display for LangString {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.value.fmt(f)
	}
}

impl RdfDisplay for LangString {
	fn rdf_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}@{}", self.value, self.language)
	}
}

impl Deref for LangString {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		self.as_str()
	}
}

impl AsRef<str> for LangString {
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

impl Borrow<str> for LangString {
	fn borrow(&self) -> &str {
		self.as_str()
	}
}
