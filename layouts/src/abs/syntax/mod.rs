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

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
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
