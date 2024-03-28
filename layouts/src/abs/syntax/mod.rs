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

pub enum Error {
	Unexpected {
		offset: usize,
		expected: json_syntax::Kind,
		found: json_syntax::Kind
	},
	DuplicateEntry {
		offset: usize,
		key: String,
		other_offset: usize
	},
	MissingType(usize),
	InvalidType {
		offset: usize,
		expected: &'static str,
		found: String
	}
}

impl Error {
	pub fn duplicate<'a>(key: &str) -> impl '_ + FnOnce(json_syntax::object::Duplicate<json_syntax::code_map::Mapped<&'a json_syntax::Value>>) -> Self {
		move |e| {
			Self::DuplicateEntry { offset: e.0.offset, key: key.to_owned(), other_offset: e.1.offset }
		}
	}
}

pub(crate) fn expect_object(
	json: &json_syntax::Value,
	offset: usize,
) -> Result<&json_syntax::Object, Error> {
	match json {
		json_syntax::Value::Object(object) => {
			Ok(object)
		}
		json => Err(Error::Unexpected {
			offset,
			expected: json_syntax::Kind::Object,
			found: json.kind()
		})
	}
}

pub(crate) fn get_entry<'a, T: json_syntax::TryFromJsonSyntax<Error = Error>>(
	object: &'a json_syntax::Object,
	key: &str,
	code_map: &json_syntax::CodeMap,
	offset: usize
) -> Result<Option<T>, Error> {
	let value = object.get_unique_mapped(code_map, offset, key)
		.map_err(Error::duplicate(key))?;

	match value {
		Some(value) => {
			let t = T::try_from_json_syntax_at(value.value, code_map, value.offset)?;
			Ok(Some(t))
		}
		None => Ok(None)
	}
}

pub(crate) fn check_type(object: &json_syntax::Object, expected: &'static str, code_map: &json_syntax::CodeMap, offset: usize) -> Result<(), Error> {
	let ty = object.get_unique_mapped(code_map, offset, "type")
		.map_err(Error::duplicate("type"))?
		.ok_or(Error::MissingType(offset))?;

	match ty.value {
		json_syntax::Value::String(found) => {
			if found == expected {
				Ok(())
			} else {
				Err(Error::InvalidType {
					offset: ty.offset,
					expected,
					found: found.to_string()
				})
			}
		}
		other => Err(Error::Unexpected {
			offset: ty.offset,
			expected: json_syntax::Kind::String,
			found: other.kind()
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ValueFormatOrLayout {
	Format(ValueFormat),
	Layout(LayoutRef),
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
