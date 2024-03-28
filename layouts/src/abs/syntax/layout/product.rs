use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::abs::{
	self,
	syntax::{Build, Context, Dataset, BuildError, Pattern, Scope, ValueFormatOrLayout, ValueIntro},
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
	pub fields: BTreeMap<String, Field>,
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
