use std::{
	borrow::Borrow,
	cmp::Ordering,
	convert::TryFrom,
	fmt,
	hash::{Hash, Hasher},
	ops::Deref,
};

use rdf_types::IriVocabulary;

use crate::{Id, IriIndex};

/// Name.
///
/// A name is a string that can serve as type/function/variable identifier in
/// a source code.
///
/// See [Unicode Standard Annex #31 (Unicode Identifier and Pattern Syntax)](https://www.unicode.org/reports/tr31/)
#[derive(Clone, Eq, Debug)]
pub struct Name {
	/// Normalized form (snake case).
	normalized: String,

	/// Original, preferred form.
	preferred: Option<String>,
}

impl PartialEq for Name {
	fn eq(&self, other: &Name) -> bool {
		self.normalized.eq(&other.normalized)
	}
}

impl PartialOrd for Name {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		self.normalized.partial_cmp(&other.normalized)
	}
}

impl Ord for Name {
	fn cmp(&self, other: &Self) -> Ordering {
		self.normalized.cmp(&other.normalized)
	}
}

impl Hash for Name {
	fn hash<H: Hasher>(&self, h: &mut H) {
		self.normalized.hash(h)
	}
}

impl Deref for Name {
	type Target = str;

	fn deref(&self) -> &str {
		self.as_str()
	}
}

impl Borrow<str> for Name {
	fn borrow(&self) -> &str {
		self.as_str()
	}
}

#[derive(Debug)]
pub struct InvalidName;

fn is_word_start(prev: Option<char>, c: char, next: Option<char>) -> bool {
	c.is_uppercase()
		&& prev.map(|p| p.is_lowercase()).unwrap_or(true)
		&& next.map(|n| n.is_lowercase()).unwrap_or(true)
}

/// Normalizes a name to snake case.
fn normalize(id: &str) -> Result<String, InvalidName> {
	let mut result = String::new();
	let mut prev = None;
	let mut boundary = true;
	let mut chars = id.chars().peekable();

	while let Some(c) = chars.next() {
		match c {
			'_' | ' ' | '-' | '.' | ',' | '/' => {
				if !boundary {
					boundary = true
				}
			}
			c if c.is_alphanumeric() => {
				if is_word_start(prev, c, chars.peek().cloned()) {
					boundary = true
				}

				if boundary {
					if !result.is_empty() {
						result.push('_');
					}
					boundary = false
				}

				result.push(c.to_lowercase().next().unwrap());
			}
			_ => (),
		}

		prev = Some(c);
	}

	if result.is_empty() {
		return Err(InvalidName);
	}

	Ok(result)
}

impl<'a> TryFrom<&'a str> for Name {
	type Error = InvalidName;

	fn try_from(id: &'a str) -> Result<Self, Self::Error> {
		Ok(Self {
			normalized: normalize(id)?,
			preferred: Some(id.to_string()),
		})
	}
}

impl TryFrom<String> for Name {
	type Error = InvalidName;

	fn try_from(id: String) -> Result<Self, Self::Error> {
		Ok(Self {
			normalized: normalize(&id)?,
			preferred: Some(id),
		})
	}
}

impl Name {
	pub fn new<S: AsRef<str>>(id: S) -> Result<Self, InvalidName> {
		Ok(Self {
			normalized: normalize(id.as_ref())?,
			preferred: Some(id.as_ref().into()),
		})
	}

	pub fn from_id(
		vocabulary: &impl IriVocabulary<Iri = IriIndex>,
		id: Id,
	) -> Result<Option<Self>, InvalidName> {
		match id {
			Id::Iri(i) => match vocabulary.iri(&i) {
				Some(iri) => Self::from_iri(iri),
				None => Ok(None),
			},
			_ => Ok(None),
		}
	}

	pub fn from_iri(iri: iref::Iri) -> Result<Option<Self>, InvalidName> {
		match iri.fragment() {
			Some(fragment) => Ok(Some(Self::new(fragment.as_str())?)),
			None => iri
				.path()
				.file_name()
				.map(|name| match std::path::Path::new(name).file_stem() {
					Some(stem) => Name::new(stem.to_string_lossy()),
					None => Name::new(name),
				})
				.transpose(),
		}
	}

	pub fn preferred(&self) -> Option<&str> {
		self.preferred.as_deref()
	}

	pub fn as_str(&self) -> &str {
		match self.preferred.as_deref() {
			Some(p) => p,
			None => &self.normalized,
		}
	}

	/// Converts this name into a snake-cased identifier.
	///
	/// ## Example
	///
	/// ```
	/// # use treeldr::Name;
	/// let name = Name::new("File_not_FoundException").unwrap();
	/// assert_eq!(name.to_snake_case(), "file_not_found_exception")
	/// ```
	pub fn to_snake_case(&self) -> String {
		self.normalized.clone()
	}

	/// Converts this name into a camel-cased identifier.
	///
	/// ## Example
	///
	/// ```
	/// # use treeldr::Name;
	/// let name = Name::new("File_not_FoundException").unwrap();
	/// assert_eq!(name.to_camel_case(), "fileNotFoundException")
	/// ```
	pub fn to_camel_case(&self) -> String {
		let segments = self.normalized.split('_').enumerate().map(|(i, segment)| {
			if i > 0 {
				let c = segment.chars().next().unwrap(); // segment is never empty.
				let (_, rest) = segment.split_at(c.len_utf8());
				let mut pascal_case_segment = c.to_uppercase().next().unwrap().to_string();
				pascal_case_segment.push_str(rest);
				pascal_case_segment
			} else {
				segment.to_string()
			}
		});

		let mut result = String::new();
		for segment in segments {
			result.push_str(&segment)
		}
		result
	}

	/// Converts this name into a pascal-cased identifier.
	///
	/// ## Example
	///
	/// ```
	/// # use treeldr::Name;
	/// let name = Name::new("File_not_FoundException").unwrap();
	/// assert_eq!(name.to_pascal_case(), "FileNotFoundException")
	/// ```
	pub fn to_pascal_case(&self) -> String {
		let segments = self.normalized.split('_').map(|segment| {
			let c = segment.chars().next().unwrap(); // segment is never empty.
			let (_, rest) = segment.split_at(c.len_utf8());
			let mut pascal_case_segment = c.to_uppercase().next().unwrap().to_string();
			pascal_case_segment.push_str(rest);
			pascal_case_segment
		});
		let mut result = String::new();
		for segment in segments {
			result.push_str(&segment)
		}
		result
	}

	/// Converts this name into a kebab-cased identifier.
	///
	/// ## Example
	///
	/// ```
	/// # use treeldr::Name;
	/// let name = Name::new("File_not_FoundException").unwrap();
	/// assert_eq!(name.to_kebab_case(), "file-not-found-exception")
	/// ```
	pub fn to_kebab_case(&self) -> String {
		let segments = self.normalized.split('_');
		let mut result = String::new();
		for (i, segment) in segments.into_iter().enumerate() {
			if i > 0 {
				result.push('-');
			}
			result.push_str(segment)
		}
		result
	}

	pub fn push(&mut self, id: &str) {
		if let Ok(id) = normalize(id) {
			self.normalized.push('_');
			self.normalized.push_str(&id);
			self.preferred = None
		}
	}

	pub fn push_name(&mut self, id: &Name) {
		self.normalized.push('_');
		self.normalized.push_str(&id.normalized);
		self.preferred = None
	}
}

impl fmt::Display for Name {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match &self.preferred {
			Some(id) => id.fmt(f),
			None => self.normalized.fmt(f),
		}
	}
}
