use std::hash::Hash;

use crate::{module::ItemPathBuf, rdf::RdfContextMut, syntax::ast, ItemPathSegment, Registry};

use super::{Build, BuildContext, BuildingPackage, Error, Scope};

impl<'ast, P, R: Clone + Ord + Hash> Build<'ast, P, R> for ast::ItemPath {
	type Target = ItemPathBuf;

	fn build<E, D>(
		&'ast self,
		_context: &mut BuildContext<'_, E, D>,
		_namespace: &BuildingPackage<'ast, P>,
		_scope: &Scope<P>,
	) -> Result<Self::Target, Error<P>>
	where
		E: ?Sized + Registry<PackageId = P, Resource = R>,
		D: RdfContextMut<Resource = R>,
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
