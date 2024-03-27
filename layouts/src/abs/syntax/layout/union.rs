use serde::{Deserialize, Serialize};

use crate::{
	abs::syntax::{Build, Context, Error, Scope},
	layout::LayoutType,
	Ref,
};

use super::{LayoutHeader, UnionLayoutType};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnionLayout {
	#[serde(rename = "type")]
	pub type_: UnionLayoutType,

	#[serde(flatten)]
	pub header: LayoutHeader,
}

impl<C: Context> Build<C> for UnionLayout {
	type Target = Vec<Ref<LayoutType, C::Resource>>;

	fn build(&self, _context: &mut C, _scope: &Scope) -> Result<Self::Target, Error> {
		unimplemented!()
	}
}
