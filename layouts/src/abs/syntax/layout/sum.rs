use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::abs::{
	self,
	syntax::{Build, Context, Dataset, Error, OneOrMany, Pattern, Scope, VariableName},
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Variant {
	#[serde(default, skip_serializing_if = "OneOrMany::is_empty")]
	pub intro: OneOrMany<String>,

	pub value: VariantFormatOrLayout,

	#[serde(default, skip_serializing_if = "Dataset::is_empty")]
	pub dataset: Dataset,
}

impl<C: Context> Build<C> for SumLayout
where
	C::Resource: Clone,
{
	type Target = abs::layout::SumLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
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

impl<C: Context> Build<C> for VariantFormatOrLayout
where
	C::Resource: Clone,
{
	type Target = crate::ValueFormat<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
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

impl<C: Context> Build<C> for VariantFormat
where
	C::Resource: Clone,
{
	type Target = crate::ValueFormat<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
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
