use std::hash::Hash;

use crate::{rdf::RdfContextMut, syntax::ast, ItemPathBuf, ItemPathSegment};

use super::{Build, BuildContext, BuildingNamespace, Error, Scope};

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::ItemPath {
	type Target = ItemPathBuf;

	fn build<D>(
		&'ast self,
		_context: &mut BuildContext<'_, R, D>,
		_namespace: &BuildingNamespace<'ast>,
		_scope: &Scope,
	) -> Result<Self::Target, Error>
	where
		D: RdfContextMut<R>,
	{
		let mut result = Vec::new();
		self.build_into(&mut result);
		Ok(ItemPathBuf(result))
	}
}

impl ast::ItemPathSegment {
	fn build(&self) -> ItemPathSegment {
		match self {
			Self::Super(_) => ItemPathSegment::Super,
			Self::Ident(i) => ItemPathSegment::Item(i.as_str().to_owned()),
		}
	}
}

impl ast::ItemPath {
	fn build_into(&self, result: &mut Vec<ItemPathSegment>) {
		result.push(self.first.build());
		if let Some(rest) = &self.rest {
			rest.rest.build_into(result);
		}
	}
}
