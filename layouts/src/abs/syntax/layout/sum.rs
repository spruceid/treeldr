use std::collections::BTreeMap;

use json_syntax::{Kind, TryFromJson, TryFromJsonObject};
use serde::{Deserialize, Serialize};

use crate::abs::{
	self,
	syntax::{
		check_type, expect_object, get_entry, require_entry, Build, BuildError, Context, Dataset,
		Error, OneOrMany, Pattern, Scope, VariableName,
	},
};

use super::{LayoutHeader, LayoutRef, SumLayoutType};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SumLayout {
	#[serde(rename = "type")]
	pub type_: SumLayoutType,

	#[serde(flatten)]
	pub header: LayoutHeader,

	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	pub variants: BTreeMap<String, Variant>,
}

impl TryFromJson for SumLayout {
	type Error = Error;

	fn try_from_json_at(
		json: &json_syntax::Value,
		code_map: &json_syntax::CodeMap,
		offset: usize,
	) -> Result<Self, Self::Error> {
		let object = expect_object(json, offset)?;
		Self::try_from_json_object_at(object, code_map, offset)
	}
}

impl TryFromJsonObject for SumLayout {
	type Error = Error;

	fn try_from_json_object_at(
		object: &json_syntax::Object,
		code_map: &json_syntax::CodeMap,
		offset: usize,
	) -> Result<Self, Self::Error> {
		check_type(object, SumLayoutType::NAME, code_map, offset)?;
		Ok(Self {
			type_: SumLayoutType,
			header: LayoutHeader::try_from_json_object_at(object, code_map, offset)?,
			variants: get_entry(object, "variants", code_map, offset)?.unwrap_or_default(),
		})
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Variant {
	#[serde(default, skip_serializing_if = "OneOrMany::is_empty")]
	pub intro: OneOrMany<String>,

	pub value: VariantFormatOrLayout,

	#[serde(default, skip_serializing_if = "Dataset::is_empty")]
	pub dataset: Dataset,
}

impl TryFromJson for Variant {
	type Error = Error;

	fn try_from_json_at(
		json: &json_syntax::Value,
		code_map: &json_syntax::CodeMap,
		offset: usize,
	) -> Result<Self, Self::Error> {
		let object = expect_object(json, offset)?;
		Self::try_from_json_object_at(object, code_map, offset)
	}
}

impl TryFromJsonObject for Variant {
	type Error = Error;

	fn try_from_json_object_at(
		object: &json_syntax::Object,
		code_map: &json_syntax::CodeMap,
		offset: usize,
	) -> Result<Self, Self::Error> {
		Ok(Self {
			intro: get_entry(object, "intro", code_map, offset)?.unwrap_or_default(),
			value: require_entry(object, "value", code_map, offset)?,
			dataset: get_entry(object, "dataset", code_map, offset)?.unwrap_or_default(),
		})
	}
}

impl<C: Context> Build<C> for SumLayout
where
	C::Resource: Clone,
{
	type Target = abs::layout::SumLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, BuildError> {
		let (header, scope) = self.header.build(context, scope)?;

		let mut variants = Vec::with_capacity(self.variants.len());

		for (name, variant) in &self.variants {
			let scope = scope.with_intro(variant.intro.as_slice())?;
			variants.push(crate::layout::sum::Variant {
				name: name.to_owned(),
				intro: variant.intro.len() as u32,
				value: variant.value.build(context, &scope)?,
				dataset: variant.dataset.build(context, &scope)?,
			})
		}

		Ok(abs::layout::SumLayout {
			input: header.input,
			intro: header.intro,
			variants,
			dataset: header.dataset,
			extra_properties: header.properties,
		})
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VariantFormatOrLayout {
	Format(VariantFormat),
	Layout(LayoutRef),
}

impl TryFromJson for VariantFormatOrLayout {
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
					VariantFormat::try_from_json_at(json, code_map, offset).map(Self::Format)
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

impl<C: Context> Build<C> for VariantFormatOrLayout
where
	C::Resource: Clone,
{
	type Target = crate::ValueFormat<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, BuildError> {
		match self {
			Self::Format(f) => f.build(context, scope),
			Self::Layout(layout) => Ok(crate::ValueFormat {
				layout: layout.build(context, scope)?,
				input: vec![Pattern::Var(VariableName::SELF.to_owned()).build(context, scope)?],
				graph: None,
			}),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VariantFormat {
	pub layout: LayoutRef,

	#[serde(default, skip_serializing_if = "VariantInput::is_default")]
	pub input: VariantInput,

	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub graph: Option<Option<Pattern>>,
}

impl TryFromJson for VariantFormat {
	type Error = Error;

	fn try_from_json_at(
		json: &json_syntax::Value,
		code_map: &json_syntax::CodeMap,
		offset: usize,
	) -> Result<Self, Self::Error> {
		let object = expect_object(json, offset)?;
		Self::try_from_json_object_at(object, code_map, offset)
	}
}

impl TryFromJsonObject for VariantFormat {
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

impl<C: Context> Build<C> for VariantFormat
where
	C::Resource: Clone,
{
	type Target = crate::ValueFormat<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, BuildError> {
		let mut inputs = Vec::with_capacity(self.input.len());
		for i in self.input.as_slice() {
			inputs.push(i.build(context, scope)?);
		}

		Ok(crate::ValueFormat {
			layout: self.layout.build(context, scope)?,
			input: inputs,
			graph: self
				.graph
				.as_ref()
				.map(|g| g.as_ref().map(|g| g.build(context, scope)).transpose())
				.transpose()?,
		})
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VariantInput(OneOrMany<Pattern>);

impl VariantInput {
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	pub fn len(&self) -> usize {
		self.0.len()
	}

	pub fn as_slice(&self) -> &[Pattern] {
		self.0.as_slice()
	}

	pub fn is_default(&self) -> bool {
		let slice = self.0.as_slice();
		slice.len() == 1 && slice[0].is_variable("self")
	}
}

impl Default for VariantInput {
	fn default() -> Self {
		Self(OneOrMany::One(Pattern::Var(VariableName::SELF.to_owned())))
	}
}

impl From<Vec<Pattern>> for VariantInput {
	fn from(value: Vec<Pattern>) -> Self {
		Self(value.into())
	}
}

impl TryFromJson for VariantInput {
	type Error = Error;

	fn try_from_json_at(
		json: &json_syntax::Value,
		code_map: &json_syntax::CodeMap,
		offset: usize,
	) -> Result<Self, Self::Error> {
		OneOrMany::try_from_json_at(json, code_map, offset).map(Self)
	}
}
