use std::collections::BTreeMap;

use json_syntax::{TryFromJson, TryFromJsonObject};
use serde::{Deserialize, Serialize};

use crate::{
	abs::{
		self,
		syntax::{
			check_type, expect_object, get_entry, require_entry, Build, BuildError, Context,
			Dataset, Error, ObjectUnusedEntries, Pattern, Scope, ValueFormatOrLayout, ValueIntro,
		},
	},
	Value,
};

use super::{LayoutHeader, ProductLayoutType};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductLayout {
	#[serde(rename = "type")]
	pub type_: ProductLayoutType,

	#[serde(flatten)]
	pub header: LayoutHeader,

	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	pub fields: BTreeMap<Value, Field>,
}

impl TryFromJson for ProductLayout {
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

impl TryFromJsonObject for ProductLayout {
	type Error = Error;

	fn try_from_json_object_at(
		object: &json_syntax::Object,
		code_map: &json_syntax::CodeMap,
		offset: usize,
	) -> Result<Self, Self::Error> {
		let mut unused_entries = ObjectUnusedEntries::new(object, code_map, offset);
		check_type(
			object,
			ProductLayoutType::NAME,
			&mut unused_entries,
			code_map,
			offset,
		)?;
		let fields: BTreeMap<String, Field> =
			get_entry(object, "fields", &mut unused_entries, code_map, offset)?.unwrap_or_default();
		let result = Self {
			type_: ProductLayoutType,
			header: LayoutHeader::try_from_json_object_at(
				object,
				&mut unused_entries,
				code_map,
				offset,
			)?,
			fields: fields
				.into_iter()
				.map(|(k, v)| (Value::string(k), v))
				.collect(),
		};
		unused_entries.check()?;
		Ok(result)
	}
}

impl<C: Context> Build<C> for ProductLayout
where
	C::Resource: Clone,
{
	type Target = abs::layout::ProductLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, BuildError> {
		let (header, scope) = self.header.build(context, scope)?;

		let mut fields = BTreeMap::new();

		for (name, field) in &self.fields {
			let scope = scope.with_intro(field.intro.as_slice())?;

			let mut dataset = field.dataset.build(context, &scope)?;

			if let Some(property) = &field.property {
				if self.header.input.is_empty() {
					return Err(BuildError::NoPropertySubject);
				} else {
					let subject = crate::Pattern::Var(0);
					if field.intro.is_empty() {
						return Err(BuildError::NoPropertyObject);
					} else {
						let object = crate::Pattern::Var(
							(self.header.input.len() + self.header.intro.len()) as u32,
						);
						let predicate = property.build(context, &scope)?;
						dataset.insert(rdf_types::Quad(subject, predicate, object, None));
					}
				}
			}

			fields.insert(
				name.to_owned(),
				crate::layout::product::Field {
					intro: field.intro.len() as u32,
					value: field.value.build(context, &scope)?,
					dataset,
					required: field.required,
				},
			);
		}

		Ok(abs::layout::ProductLayout {
			input: header.input,
			intro: header.intro,
			fields,
			dataset: header.dataset,
			extra_properties: header.properties,
		})
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Field {
	#[serde(default, skip_serializing_if = "ValueIntro::is_default")]
	pub intro: ValueIntro,

	pub value: ValueFormatOrLayout,

	#[serde(default, skip_serializing_if = "Dataset::is_empty")]
	pub dataset: Dataset,

	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub property: Option<Pattern>,

	#[serde(default, skip_serializing_if = "crate::abs::is_false")]
	pub required: bool,
}

impl TryFromJson for Field {
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

impl TryFromJsonObject for Field {
	type Error = Error;

	fn try_from_json_object_at(
		object: &json_syntax::Object,
		code_map: &json_syntax::CodeMap,
		offset: usize,
	) -> Result<Self, Self::Error> {
		let mut unused_entries = ObjectUnusedEntries::new(object, code_map, offset);
		let result = Self {
			intro: get_entry(object, "intro", &mut unused_entries, code_map, offset)?
				.unwrap_or_default(),
			value: require_entry(object, "value", &mut unused_entries, code_map, offset)?,
			dataset: get_entry(object, "dataset", &mut unused_entries, code_map, offset)?
				.unwrap_or_default(),
			property: get_entry(object, "property", &mut unused_entries, code_map, offset)?,
			required: get_entry(object, "required", &mut unused_entries, code_map, offset)?
				.unwrap_or_default(),
		};
		unused_entries.check()?;
		Ok(result)
	}
}
