use serde::{Deserialize, Serialize};

use crate::{
	abs::syntax::{Build, Context, BuildError, Scope},
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

impl<C: Context> Build<C> for IntersectionLayout {
	type Target = Vec<Ref<LayoutType, C::Resource>>;

	fn build(&self, _context: &mut C, _scope: &Scope) -> Result<Self::Target, BuildError> {
		unimplemented!()
	}
}
