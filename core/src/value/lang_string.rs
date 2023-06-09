use rdf_types::{vocabulary::LanguageTagIndex, RdfDisplayWithContext, LanguageTagVocabulary};
use std::{borrow::Borrow, fmt, ops::Deref};

/// Language tagged string.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LangString {
	value: String,
	language: LanguageTagIndex,
}

impl LangString {
	pub fn new(value: String, language: LanguageTagIndex) -> Self {
		Self { value, language }
	}

	pub fn as_str(&self) -> &str {
		&self.value
	}

	pub fn language(&self) -> LanguageTagIndex {
		self.language
	}
}

impl fmt::Display for LangString {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.value.fmt(f)
	}
}

impl<V: LanguageTagVocabulary<LanguageTag = LanguageTagIndex>> RdfDisplayWithContext<V> for LangString {
	fn rdf_fmt_with(&self, vocabulary: &V, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}@{}", self.value, vocabulary.language_tag(&self.language).unwrap())
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
