use std::collections::BTreeMap;

use crate::TypeRef;

use super::Expr;

/// Match expressions.
///
/// ```tldr
/// match {
///   foo: ?x where { ... } {
///     // ...
///   },
///   bar: ?y where { ... } {
///     // ...
///   }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Match<R, T = TypeRef> {
	pub cases: BTreeMap<String, Expr<R, T>>,

	pub order: Vec<String>,
}

impl<R: Ord, T> Match<R, T> {
	pub fn map_types<U: Ord>(self, mut f: impl FnMut(T) -> U) -> Match<R, U> {
		self.map_types_ref_mut(&mut f)
	}

	pub fn map_types_ref_mut<U: Ord>(self, f: &mut impl FnMut(T) -> U) -> Match<R, U> {
		Match {
			cases: self
				.cases
				.into_iter()
				.map(|(k, e)| (k, e.map_types_ref_mut(f)))
				.collect(),
			order: self.order,
		}
	}
}
