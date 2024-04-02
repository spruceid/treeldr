use core::fmt;

use json_syntax::{array::JsonArray, Kind, TryFromJson, TryFromJsonObject};
use serde::{Deserialize, Serialize};

mod build;
mod dataset;
mod layout;
mod pattern;
mod resource;

pub use build::*;
pub use dataset::*;
pub use layout::*;
pub use pattern::*;
pub use resource::*;

use super::regexp;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("expected {expected}, found {found}")]
	Unexpected {
		offset: usize,
		expected: json_syntax::KindSet,
		found: json_syntax::Kind,
	},

	#[error("missing required entry `{key}`")]
	MissingRequiredEntry { offset: usize, key: String },

	#[error("duplicate entry `{key}`")]
	DuplicateEntry {
		offset: usize,
		key: String,
		other_offset: usize,
	},

	#[error("missing `type` entry")]
	MissingType(usize),

	#[error("invalid type value")]
	InvalidType {
		offset: usize,
		expected: ExpectedType,
		found: String,
	},

	#[error("invalid regular expression: {0}")]
	InvalidRegex(usize, regexp::ParseError),

	#[error("invalid IRI or blank node identifier `{0}`")]
	InvalidPattern(usize, String),

	#[error("invalid language tag `{0}`")]
	InvalidLangTag(usize, String),

	#[error("invalid compact IRI `{0}`")]
	InvalidCompactIri(usize, String),

	#[error("quad must have at least 3 patterns (subject, predicate, object)")]
	MissingQuadPattern(usize),

	#[error("quad must have at most 4 patterns (subject, predicate, object, graph)")]
	TooManyQuadPatterns(usize),

	#[error("expected integer number, found {0}")]
	ExpectedInteger(usize, json_syntax::NumberBuf),

	#[error("integer number is too large: {0}")]
	IntegerOverflow(usize, json_syntax::NumberBuf),
}

impl Error {
	pub fn duplicate<'a>(
		key: &str,
	) -> impl '_ + FnOnce(json_syntax::object::Duplicate<json_syntax::object::MappedEntry<'a>>) -> Self
	{
		move |e| Self::DuplicateEntry {
			offset: e.0.value.key.offset,
			key: key.to_owned(),
			other_offset: e.1.value.key.offset,
		}
	}

	pub fn position(&self) -> usize {
		match self {
			Self::Unexpected { offset, .. } => *offset,
			Self::MissingRequiredEntry { offset, .. } => *offset,
			Self::DuplicateEntry { offset, .. } => *offset,
			Self::MissingType(offset) => *offset,
			Self::InvalidType { offset, .. } => *offset,
			Self::InvalidRegex(offset, _) => *offset,
			Self::InvalidPattern(offset, _) => *offset,
			Self::InvalidLangTag(offset, _) => *offset,
			Self::InvalidCompactIri(offset, _) => *offset,
			Self::MissingQuadPattern(offset) => *offset,
			Self::TooManyQuadPatterns(offset) => *offset,
			Self::ExpectedInteger(offset, _) => *offset,
			Self::IntegerOverflow(offset, _) => *offset,
		}
	}

	pub fn hints(&self) -> Vec<ErrorHint> {
		match self {
			Self::DuplicateEntry { other_offset, .. } => {
				vec![ErrorHint::DuplicateEntry(*other_offset)]
			}
			Self::InvalidType {
				expected, found, ..
			} => vec![
				ErrorHint::ExpectedType(*expected),
				ErrorHint::FoundType(found),
			],
			_ => Vec::new(),
		}
	}
}

impl From<json_syntax::code_map::Mapped<InvalidCompactIri>> for Error {
	fn from(value: json_syntax::code_map::Mapped<InvalidCompactIri>) -> Self {
		Self::InvalidCompactIri(value.offset, value.value.0)
	}
}

impl From<json_syntax::code_map::Mapped<json_syntax::Unexpected>> for Error {
	fn from(value: json_syntax::code_map::Mapped<json_syntax::Unexpected>) -> Self {
		Self::Unexpected {
			offset: value.offset,
			expected: value.value.expected,
			found: value.value.found,
		}
	}
}

impl From<std::convert::Infallible> for Error {
	fn from(_value: std::convert::Infallible) -> Self {
		unreachable!()
	}
}

impl From<json_syntax::code_map::Mapped<std::convert::Infallible>> for Error {
	fn from(_value: json_syntax::code_map::Mapped<std::convert::Infallible>) -> Self {
		unreachable!()
	}
}

pub enum ErrorHint<'a> {
	ExpectedType(ExpectedType),
	FoundType(&'a str),
	DuplicateEntry(usize),
}

impl<'a> ErrorHint<'a> {
	pub fn position(&self) -> Option<usize> {
		match self {
			Self::DuplicateEntry(offset) => Some(*offset),
			_ => None,
		}
	}
}

impl<'a> fmt::Display for ErrorHint<'a> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::ExpectedType(e) => match e {
				ExpectedType::One(ty) => {
					write!(f, "only possible value is `{ty}`")
				}
				ExpectedType::Many(tys) => {
					f.write_str("possible values are ")?;
					for (i, ty) in tys.iter().enumerate() {
						if i > 0 {
							if i + 1 == tys.len() {
								f.write_str(" and ")?;
							} else {
								f.write_str(", ")?;
							}
						}

						write!(f, "`{ty}`")?;
					}

					Ok(())
				}
			},
			Self::FoundType(ty) => write!(f, "found type `{ty}`"),
			Self::DuplicateEntry(_) => write!(f, "also defined here"),
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub enum ExpectedType {
	One(&'static str),
	Many(&'static [&'static str]),
}

pub(crate) fn expect_object(
	json: &json_syntax::Value,
	offset: usize,
) -> Result<&json_syntax::Object, Error> {
	match json {
		json_syntax::Value::Object(object) => Ok(object),
		json => Err(Error::Unexpected {
			offset,
			expected: json_syntax::KindSet::OBJECT,
			found: json.kind(),
		}),
	}
}

pub(crate) fn expect_array(
	json: &json_syntax::Value,
	offset: usize,
) -> Result<&[json_syntax::Value], Error> {
	match json {
		json_syntax::Value::Array(value) => Ok(value),
		json => Err(Error::Unexpected {
			offset,
			expected: json_syntax::KindSet::ARRAY,
			found: json.kind(),
		}),
	}
}

pub(crate) fn expect_string(json: &json_syntax::Value, offset: usize) -> Result<&str, Error> {
	match json {
		json_syntax::Value::String(value) => Ok(value),
		json => Err(Error::Unexpected {
			offset,
			expected: json_syntax::KindSet::STRING,
			found: json.kind(),
		}),
	}
}

pub(crate) fn get_entry<T: json_syntax::TryFromJson>(
	object: &json_syntax::Object,
	key: &str,
	code_map: &json_syntax::CodeMap,
	offset: usize,
) -> Result<Option<T>, Error>
where
	T::Error: Into<Error>,
{
	let entry = object
		.get_unique_mapped_entry(code_map, offset, key)
		.map_err(Error::duplicate(key))?;

	match entry {
		Some(entry) => {
			let t =
				T::try_from_json_at(entry.value.value.value, code_map, entry.value.value.offset)
					.map_err(Into::into)?;
			Ok(Some(t))
		}
		None => Ok(None),
	}
}

pub(crate) fn require_entry<T: json_syntax::TryFromJson>(
	object: &json_syntax::Object,
	key: &str,
	code_map: &json_syntax::CodeMap,
	offset: usize,
) -> Result<T, Error>
where
	T::Error: Into<Error>,
{
	let entry = object
		.get_unique_mapped_entry(code_map, offset, key)
		.map_err(Error::duplicate(key))?;

	match entry {
		Some(entry) => {
			T::try_from_json_at(entry.value.value.value, code_map, entry.value.value.offset)
				.map_err(Into::into)
		}
		None => Err(Error::MissingRequiredEntry {
			offset,
			key: key.to_owned(),
		}),
	}
}

pub(crate) fn require_type<'a>(
	object: &'a json_syntax::Object,
	code_map: &json_syntax::CodeMap,
	offset: usize,
) -> Result<json_syntax::code_map::Mapped<&'a str>, Error> {
	let entry = object
		.get_unique_mapped_entry(code_map, offset, "type")
		.map_err(Error::duplicate("type"))?
		.ok_or(Error::MissingType(offset))?;

	match entry.value.value.value {
		json_syntax::Value::String(found) => Ok(json_syntax::code_map::Mapped::new(
			entry.value.value.offset,
			found,
		)),
		other => Err(Error::Unexpected {
			offset: entry.value.value.offset,
			expected: json_syntax::KindSet::STRING,
			found: other.kind(),
		}),
	}
}

pub(crate) fn check_type(
	object: &json_syntax::Object,
	expected: &'static str,
	code_map: &json_syntax::CodeMap,
	offset: usize,
) -> Result<(), Error> {
	let found = require_type(object, code_map, offset)?;
	if found.value == expected {
		Ok(())
	} else {
		Err(Error::InvalidType {
			offset: found.offset,
			expected: ExpectedType::One(expected),
			found: found.value.to_string(),
		})
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OneOrMany<T> {
	One(T),
	Many(Vec<T>),
}

impl<T> OneOrMany<T> {
	pub fn is_empty(&self) -> bool {
		match self {
			Self::One(_) => false,
			Self::Many(v) => v.is_empty(),
		}
	}

	pub fn len(&self) -> usize {
		match self {
			Self::One(_) => 1,
			Self::Many(v) => v.len(),
		}
	}

	pub fn as_slice(&self) -> &[T] {
		match self {
			Self::One(t) => std::slice::from_ref(t),
			Self::Many(v) => v.as_slice(),
		}
	}
}

impl<T> Default for OneOrMany<T> {
	fn default() -> Self {
		Self::Many(Vec::new())
	}
}

impl<T> From<Vec<T>> for OneOrMany<T> {
	fn from(value: Vec<T>) -> Self {
		if value.len() == 1 {
			Self::One(value.into_iter().next().unwrap())
		} else {
			Self::Many(value)
		}
	}
}

impl<T: TryFromJson> TryFromJson for OneOrMany<T> {
	type Error = T::Error;

	fn try_from_json_at(
		json: &json_syntax::Value,
		code_map: &json_syntax::CodeMap,
		offset: usize,
	) -> Result<Self, Self::Error> {
		match json {
			json_syntax::Value::Array(array) => Ok(Self::Many(
				array
					.iter_mapped(code_map, offset)
					.map(|item| T::try_from_json_at(item.value, code_map, item.offset))
					.collect::<Result<_, _>>()?,
			)),
			other => T::try_from_json_at(other, code_map, offset).map(Self::One),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ValueFormatOrLayout {
	Format(ValueFormat),
	Layout(LayoutRef),
}

impl TryFromJson for ValueFormatOrLayout {
	type Error = Error;

	fn try_from_json_at(
		json: &json_syntax::Value,
		code_map: &json_syntax::CodeMap,
		offset: usize,
	) -> Result<Self, Self::Error> {
		match json {
			json_syntax::Value::String(value) => {
				LayoutRef::try_from_json_string_at(value, offset).map(Self::Layout)
			}
			json_syntax::Value::Object(value) => {
				if value.contains_key("type") {
					LayoutRef::try_from_json_object_at(value, code_map, offset).map(Self::Layout)
				} else {
					ValueFormat::try_from_json_object_at(value, code_map, offset).map(Self::Format)
				}
			}
			other => Err(Error::Unexpected {
				offset,
				expected: Kind::String | Kind::Object,
				found: other.kind(),
			}),
		}
	}
}

impl<C: Context> Build<C> for ValueFormatOrLayout
where
	C::Resource: Clone,
{
	type Target = crate::ValueFormat<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, BuildError> {
		match self {
			Self::Format(f) => f.build(context, scope),
			Self::Layout(layout) => Ok(crate::ValueFormat {
				layout: layout.build(context, scope)?,
				input: vec![Pattern::Var(VariableName::VALUE.to_owned()).build(context, scope)?],
				graph: None,
			}),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValueFormat {
	pub layout: LayoutRef,

	#[serde(default, skip_serializing_if = "ValueInput::is_default")]
	pub input: ValueInput,

	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub graph: Option<Option<Pattern>>,
}

impl TryFromJsonObject for ValueFormat {
	type Error = Error;

	fn try_from_json_object_at(
		object: &json_syntax::Object,
		code_map: &json_syntax::CodeMap,
		offset: usize,
	) -> Result<Self, Self::Error> {
		Ok(Self {
			layout: require_entry(object, "layout", code_map, offset)?,
			input: get_entry(object, "input", code_map, offset)?.unwrap_or_default(),
			graph: get_entry(object, "graph", code_map, offset)?.unwrap_or_default(),
		})
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ValueIntro(OneOrMany<String>);

impl ValueIntro {
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	pub fn len(&self) -> usize {
		self.0.len()
	}

	pub fn as_slice(&self) -> &[String] {
		self.0.as_slice()
	}

	pub fn is_default(&self) -> bool {
		let slice = self.0.as_slice();
		slice.len() == 1 && slice[0] == "value"
	}
}

impl Default for ValueIntro {
	fn default() -> Self {
		Self(OneOrMany::One("value".to_owned()))
	}
}

impl From<Vec<String>> for ValueIntro {
	fn from(value: Vec<String>) -> Self {
		Self(value.into())
	}
}

impl TryFromJson for ValueIntro {
	type Error = Error;

	fn try_from_json_at(
		json: &json_syntax::Value,
		code_map: &json_syntax::CodeMap,
		offset: usize,
	) -> Result<Self, Self::Error> {
		Ok(Self(OneOrMany::try_from_json_at(json, code_map, offset)?))
	}
}
