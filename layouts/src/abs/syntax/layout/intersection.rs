use json_syntax::TryFromJsonObject;
use serde::{Deserialize, Serialize};

use crate::{
	abs::syntax::{check_type, Build, BuildError, Context, Error, Scope},
	layout::LayoutType,
	Ref,
};

use super::{IntersectionLayoutType, LayoutHeader};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntersectionLayout {
	#[serde(rename = "type")]
	pub type_: IntersectionLayoutType,

	#[serde(flatten)]
	pub header: LayoutHeader,
}

impl TryFromJsonObject for IntersectionLayout {
	type Error = Error;

	fn try_from_json_object_at(
		object: &json_syntax::Object,
		code_map: &json_syntax::CodeMap,
		offset: usize,
	) -> Result<Self, Self::Error> {
		check_type(object, IntersectionLayoutType::NAME, code_map, offset)?;
		Ok(Self {
			type_: IntersectionLayoutType,
			header: LayoutHeader::try_from_json_object_at(object, code_map, offset)?,
		})
	}
}

impl<C: Context> Build<C> for IntersectionLayout {
	type Target = Vec<Ref<LayoutType, C::Resource>>;

	fn build(&self, _context: &mut C, _scope: &Scope) -> Result<Self::Target, BuildError> {
		unimplemented!()
	}
}
