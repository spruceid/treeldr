use json_syntax::{TryFromJsonObject, TryFromJsonSyntax};
use serde::{Deserialize, Serialize};

use crate::{
	abs::{
		self,
		syntax::{
			check_type, expect_object, get_entry, require_entry, require_type, Build, BuildError,
			CompactIri, Context, Error, ExpectedType, Pattern, Scope,
		},
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

impl TryFromJsonObject for LiteralLayout {
	type Error = Error;

	fn try_from_json_object_at(
		object: &json_syntax::Object,
		code_map: &json_syntax::CodeMap,
		offset: usize,
	) -> Result<Self, Self::Error> {
		let ty = require_type(object, code_map, offset)?;
		match ty.value {
			IdLayoutType::NAME => {
				IdLayout::try_from_json_object_at(object, code_map, offset).map(Self::Id)
			}
			UnitLayoutType::NAME
			| BooleanLayoutType::NAME
			| NumberLayoutType::NAME
			| ByteStringLayoutType::NAME
			| TextStringLayoutType::NAME => {
				DataLayout::try_from_json_object_at(object, code_map, offset).map(Self::Data)
			}
			unexpected => Err(Error::InvalidType {
				offset: ty.offset,
				expected: ExpectedType::Many(&[
					IdLayoutType::NAME,
					UnitLayoutType::NAME,
					BooleanLayoutType::NAME,
					NumberLayoutType::NAME,
					ByteStringLayoutType::NAME,
					TextStringLayoutType::NAME,
				]),
				found: unexpected.to_owned(),
			}),
		}
	}
}

impl<C: Context> Build<C> for LiteralLayout
where
	C::Resource: Clone,
{
	type Target = abs::layout::LiteralLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, BuildError> {
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

impl TryFromJsonSyntax for DataLayout {
	type Error = Error;

	fn try_from_json_syntax_at(
		json: &json_syntax::Value,
		code_map: &json_syntax::CodeMap,
		offset: usize,
	) -> Result<Self, Error> {
		let object = expect_object(json, offset)?;
		Self::try_from_json_object_at(object, code_map, offset)
	}
}

impl TryFromJsonObject for DataLayout {
	type Error = Error;

	fn try_from_json_object_at(
		object: &json_syntax::Object,
		code_map: &json_syntax::CodeMap,
		offset: usize,
	) -> Result<Self, Error> {
		let ty = require_type(object, code_map, offset)?;
		match ty.value {
			UnitLayoutType::NAME => {
				UnitLayout::try_from_json_object_at(object, code_map, offset).map(Self::Unit)
			}
			BooleanLayoutType::NAME => {
				BooleanLayout::try_from_json_object_at(object, code_map, offset).map(Self::Boolean)
			}
			NumberLayoutType::NAME => {
				NumberLayout::try_from_json_object_at(object, code_map, offset).map(Self::Number)
			}
			ByteStringLayoutType::NAME => {
				ByteStringLayout::try_from_json_object_at(object, code_map, offset)
					.map(Self::ByteString)
			}
			TextStringLayoutType::NAME => {
				TextStringLayout::try_from_json_object_at(object, code_map, offset)
					.map(Self::TextString)
			}
			unexpected => Err(Error::InvalidType {
				offset: ty.offset,
				expected: ExpectedType::Many(&[
					UnitLayoutType::NAME,
					BooleanLayoutType::NAME,
					NumberLayoutType::NAME,
					ByteStringLayoutType::NAME,
					TextStringLayoutType::NAME,
				]),
				found: unexpected.to_owned(),
			}),
		}
	}
}

impl<C: Context> Build<C> for DataLayout
where
	C::Resource: Clone,
{
	type Target = abs::layout::DataLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, BuildError> {
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

impl TryFromJsonObject for UnitLayout {
	type Error = Error;

	fn try_from_json_object_at(
		object: &json_syntax::Object,
		code_map: &json_syntax::CodeMap,
		offset: usize,
	) -> Result<Self, Error> {
		check_type(object, UnitLayoutType::NAME, code_map, offset)?;
		let header = LayoutHeader::try_from_json_object_at(object, code_map, offset)?;
		let const_ = get_entry(object, "const", code_map, offset)?.unwrap_or_default();

		Ok(Self {
			type_: UnitLayoutType,
			header,
			const_,
		})
	}
}

impl<C: Context> Build<C> for UnitLayout
where
	C::Resource: Clone,
{
	type Target = abs::layout::UnitLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, BuildError> {
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
) -> Result<crate::Pattern<C::Resource>, BuildError> {
	match resource {
		Some(r) => r.build(context, scope),
		None => {
			if input.is_empty() {
				Err(BuildError::MissingLiteralTargetResource)
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

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, BuildError> {
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

impl TryFromJsonObject for BooleanLayout {
	type Error = Error;

	fn try_from_json_object_at(
		object: &json_syntax::Object,
		code_map: &json_syntax::CodeMap,
		offset: usize,
	) -> Result<Self, Error> {
		check_type(object, BooleanLayoutType::NAME, code_map, offset)?;
		let header = LayoutHeader::try_from_json_object_at(object, code_map, offset)?;
		let resource = get_entry(object, "resource", code_map, offset)?;
		let datatype = get_entry(object, "datatype", code_map, offset)?;

		Ok(Self {
			type_: BooleanLayoutType,
			header,
			resource,
			datatype,
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

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, BuildError> {
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

impl TryFromJsonObject for NumberLayout {
	type Error = Error;

	fn try_from_json_object_at(
		object: &json_syntax::Object,
		code_map: &json_syntax::CodeMap,
		offset: usize,
	) -> Result<Self, Error> {
		check_type(object, NumberLayoutType::NAME, code_map, offset)?;
		let header = LayoutHeader::try_from_json_object_at(object, code_map, offset)?;
		let resource = get_entry(object, "resource", code_map, offset)?;
		let datatype = require_entry(object, "datatype", code_map, offset)?;

		Ok(Self {
			type_: NumberLayoutType,
			header,
			resource,
			datatype,
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

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, BuildError> {
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

impl TryFromJsonObject for ByteStringLayout {
	type Error = Error;

	fn try_from_json_object_at(
		object: &json_syntax::Object,
		code_map: &json_syntax::CodeMap,
		offset: usize,
	) -> Result<Self, Error> {
		check_type(object, ByteStringLayoutType::NAME, code_map, offset)?;
		let header = LayoutHeader::try_from_json_object_at(object, code_map, offset)?;
		let resource = get_entry(object, "resource", code_map, offset)?;
		let datatype = require_entry(object, "datatype", code_map, offset)?;

		Ok(Self {
			type_: ByteStringLayoutType,
			header,
			resource,
			datatype,
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

impl TryFromJsonObject for TextStringLayout {
	type Error = Error;

	fn try_from_json_object_at(
		object: &json_syntax::Object,
		code_map: &json_syntax::CodeMap,
		offset: usize,
	) -> Result<Self, Error> {
		check_type(object, TextStringLayoutType::NAME, code_map, offset)?;
		let header = LayoutHeader::try_from_json_object_at(object, code_map, offset)?;
		let pattern = get_entry(object, "pattern", code_map, offset)?;
		let resource = get_entry(object, "resource", code_map, offset)?;
		let datatype = get_entry(object, "datatype", code_map, offset)?;

		Ok(Self {
			type_: TextStringLayoutType,
			header,
			pattern,
			resource,
			datatype,
		})
	}
}

impl<C: Context> Build<C> for TextStringLayout
where
	C::Resource: Clone,
{
	type Target = abs::layout::TextStringLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, BuildError> {
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

impl TryFromJsonObject for IdLayout {
	type Error = Error;

	fn try_from_json_object_at(
		object: &json_syntax::Object,
		code_map: &json_syntax::CodeMap,
		offset: usize,
	) -> Result<Self, Error> {
		check_type(object, IdLayoutType::NAME, code_map, offset)?;
		let header = LayoutHeader::try_from_json_object_at(object, code_map, offset)?;
		let pattern = get_entry(object, "pattern", code_map, offset)?;
		let resource = get_entry(object, "resource", code_map, offset)?;

		Ok(Self {
			type_: IdLayoutType,
			header,
			pattern,
			resource,
		})
	}
}

impl<C: Context> Build<C> for IdLayout
where
	C::Resource: Clone,
{
	type Target = abs::layout::IdLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, BuildError> {
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
