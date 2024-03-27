use serde::{Deserialize, Serialize};

use crate::{
	abs::{
		self,
		syntax::{Build, CompactIri, Context, Error, Pattern, Scope},
		RegExp,
	},
	Value,
};

use super::{
	BooleanLayoutType, ByteStringLayoutType, IdLayoutType, LayoutHeader, LayoutInput,
	NumberLayoutType, TextStringLayoutType, UnitLayoutType,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LiteralLayout {
	Data(DataLayout),
	Id(IdLayout),
}

impl LiteralLayout {
	pub fn id(&self) -> Option<&CompactIri> {
		match self {
			Self::Data(l) => l.id(),
			Self::Id(l) => l.header.id.as_ref(),
		}
	}
}

impl<C: Context> Build<C> for LiteralLayout
where
	C::Resource: Clone,
{
	type Target = abs::layout::LiteralLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		match self {
			Self::Data(l) => Ok(abs::layout::LiteralLayout::Data(l.build(context, scope)?)),
			Self::Id(l) => Ok(abs::layout::LiteralLayout::Id(l.build(context, scope)?)),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DataLayout {
	Unit(UnitLayout),
	Boolean(BooleanLayout),
	Number(NumberLayout),
	ByteString(ByteStringLayout),
	TextString(TextStringLayout),
}

impl DataLayout {
	pub fn id(&self) -> Option<&CompactIri> {
		match self {
			Self::Unit(l) => l.header.id.as_ref(),
			Self::Boolean(l) => l.header.id.as_ref(),
			Self::Number(l) => l.header.id.as_ref(),
			Self::ByteString(l) => l.header.id.as_ref(),
			Self::TextString(l) => l.header.id.as_ref(),
		}
	}
}

impl<C: Context> Build<C> for DataLayout
where
	C::Resource: Clone,
{
	type Target = abs::layout::DataLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		match self {
			Self::Unit(l) => l.build(context, scope).map(abs::layout::DataLayout::Unit),
			Self::Boolean(l) => l
				.build(context, scope)
				.map(abs::layout::DataLayout::Boolean),
			Self::Number(l) => l.build(context, scope).map(abs::layout::DataLayout::Number),
			Self::ByteString(l) => l
				.build(context, scope)
				.map(abs::layout::DataLayout::ByteString),
			Self::TextString(l) => l
				.build(context, scope)
				.map(abs::layout::DataLayout::TextString),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnitLayout {
	#[serde(rename = "type")]
	pub type_: UnitLayoutType,

	#[serde(flatten)]
	pub header: LayoutHeader,

	#[serde(rename = "const", default, skip_serializing_if = "Value::is_unit")]
	pub const_: Value,
}

impl<C: Context> Build<C> for UnitLayout
where
	C::Resource: Clone,
{
	type Target = abs::layout::UnitLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let (header, _) = self.header.build(context, scope)?;
		Ok(abs::layout::UnitLayout {
			input: header.input,
			intro: header.intro,
			dataset: header.dataset,
			const_: self.const_.clone(),
			extra_properties: header.properties,
		})
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BooleanLayout {
	#[serde(rename = "type")]
	pub type_: BooleanLayoutType,

	#[serde(flatten)]
	pub header: LayoutHeader,

	pub resource: Option<Pattern>,

	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub datatype: Option<CompactIri>,
}

fn literal_resource<C: Context>(
	context: &mut C,
	scope: &Scope,
	input: &LayoutInput,
	resource: Option<&Pattern>,
) -> Result<crate::Pattern<C::Resource>, Error> {
	match resource {
		Some(r) => r.build(context, scope),
		None => {
			if input.is_empty() {
				Err(Error::MissingLiteralTargetResource)
			} else {
				Ok(crate::Pattern::Var(0))
			}
		}
	}
}

impl<C: Context> Build<C> for BooleanLayout
where
	C::Resource: Clone,
{
	type Target = abs::layout::BooleanLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let (header, scope) = self.header.build(context, scope)?;

		Ok(abs::layout::BooleanLayout {
			input: header.input,
			intro: header.intro,
			dataset: header.dataset,
			resource: literal_resource(
				context,
				&scope,
				&self.header.input,
				self.resource.as_ref(),
			)?,
			datatype: self
				.datatype
				.as_ref()
				.map(|i| i.build(context, &scope))
				.transpose()?
				.unwrap_or_else(|| context.iri_resource(xsd_types::XSD_BOOLEAN)),
			extra_properties: header.properties,
		})
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NumberLayout {
	#[serde(rename = "type")]
	pub type_: NumberLayoutType,

	#[serde(flatten)]
	pub header: LayoutHeader,

	pub resource: Option<Pattern>,

	pub datatype: CompactIri,
}

impl<C: Context> Build<C> for NumberLayout
where
	C::Resource: Clone,
{
	type Target = abs::layout::NumberLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let (header, scope) = self.header.build(context, scope)?;

		Ok(abs::layout::NumberLayout {
			input: header.input,
			intro: header.intro,
			dataset: header.dataset,
			resource: literal_resource(
				context,
				&scope,
				&self.header.input,
				self.resource.as_ref(),
			)?,
			datatype: self.datatype.build(context, &scope)?,
			extra_properties: header.properties,
		})
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ByteStringLayout {
	#[serde(rename = "type")]
	pub type_: ByteStringLayoutType,

	#[serde(flatten)]
	pub header: LayoutHeader,

	pub resource: Option<Pattern>,

	pub datatype: CompactIri,
}

impl<C: Context> Build<C> for ByteStringLayout
where
	C::Resource: Clone,
{
	type Target = abs::layout::ByteStringLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let (header, scope) = self.header.build(context, scope)?;

		Ok(abs::layout::ByteStringLayout {
			input: header.input,
			intro: header.intro,
			dataset: header.dataset,
			resource: literal_resource(
				context,
				&scope,
				&self.header.input,
				self.resource.as_ref(),
			)?,
			datatype: self.datatype.build(context, &scope)?,
			extra_properties: header.properties,
		})
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextStringLayout {
	#[serde(rename = "type")]
	pub type_: TextStringLayoutType,

	#[serde(flatten)]
	pub header: LayoutHeader,

	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub pattern: Option<RegExp>,

	pub resource: Option<Pattern>,

	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub datatype: Option<CompactIri>,
}

impl<C: Context> Build<C> for TextStringLayout
where
	C::Resource: Clone,
{
	type Target = abs::layout::TextStringLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let (header, scope) = self.header.build(context, scope)?;

		Ok(abs::layout::TextStringLayout {
			input: header.input,
			intro: header.intro,
			dataset: header.dataset,
			pattern: self.pattern.clone(),
			resource: literal_resource(
				context,
				&scope,
				&self.header.input,
				self.resource.as_ref(),
			)?,
			datatype: self
				.datatype
				.as_ref()
				.map(|i| i.build(context, &scope))
				.transpose()?
				.unwrap_or_else(|| context.iri_resource(xsd_types::XSD_STRING)),
			properties: header.properties,
		})
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IdLayout {
	#[serde(rename = "type")]
	pub type_: IdLayoutType,

	#[serde(flatten)]
	pub header: LayoutHeader,

	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub pattern: Option<RegExp>,

	pub resource: Option<Pattern>,
}

impl<C: Context> Build<C> for IdLayout
where
	C::Resource: Clone,
{
	type Target = abs::layout::IdLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let (header, scope) = self.header.build(context, scope)?;

		Ok(abs::layout::IdLayout {
			input: header.input,
			intro: header.intro,
			dataset: header.dataset,
			pattern: self.pattern.clone(),
			resource: literal_resource(
				context,
				&scope,
				&self.header.input,
				self.resource.as_ref(),
			)?,
			properties: header.properties,
		})
	}
}
