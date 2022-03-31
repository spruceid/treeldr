use std::{
	cmp::Ordering,
	convert::TryFrom,
	fmt,
	hash::{Hash, Hasher},
};

/// Name.
/// 
/// A name is a string that can serve as type/function/variable identifier in
/// a source code.
/// 
/// See [Unicode Standard Annex #31 (Unicode Identifier and Pattern Syntax)](https://www.unicode.org/reports/tr31/)
#[derive(Clone, Eq, Debug)]
pub struct Name {
	/// Normalized form (caml case).
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

#[derive(Debug)]
pub struct InvalidName;

fn is_word_start(prev: Option<char>, c: char, next: Option<char>) -> bool {
	c.is_uppercase()
		&& prev.map(|p| p.is_lowercase()).unwrap_or(true)
		&& next.map(|n| n.is_lowercase()).unwrap_or(true)
}

fn normalize(id: &str) -> Result<String, InvalidName> {
	let mut result = String::new();
	let mut prev = None;
	let mut boundary = true;
	let mut separated = true;
	let mut chars = id.chars().peekable();

	while let Some(c) = chars.next() {
		match c {
			c if c.is_digit(10) && result.is_empty() => break,
			'%' => {
				// authorized in first position.
				if !result.is_empty() {
					break;
				}
			}
			'_' | ' ' | '-' => {
				if !boundary {
					boundary = true;
					separated = false
				}
			}
			c if c.is_alphanumeric() => {
				if is_word_start(prev, c, chars.peek().cloned()) {
					boundary = true
				}

				if boundary && !separated {
					result.push('_');
					separated = true
				}

				result.push(c.to_lowercase().next().unwrap());
				boundary = false
			}
			_ => {
				return Err(InvalidName);
			}
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
			preferred: None,
		})
	}

	pub fn as_str(&self) -> &str {
		&self.normalized
	}

	pub fn to_snake_case(&self) -> String {
		self.normalized.clone()
	}

	pub fn to_caml_case(&self) -> String {
		let segments = self.normalized
			.split('_')
			.map(|segment| {
				let c = segment.chars().next().unwrap(); // segment is never empty.
				let (_, rest) = segment.split_at(c.len_utf8());
				IntoIterator::into_iter([
					c.to_uppercase().next().unwrap().to_string(),
					rest.to_string(),
				])
			})
			.flatten();
		let mut result = String::new();
		for segment in segments {
			result.push_str(&segment)
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

	pub fn push_ident(&mut self, id: &Name) {
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
