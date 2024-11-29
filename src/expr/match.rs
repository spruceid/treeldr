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
pub struct Match<P, R, T = TypeRef<P>> {
	pub cases: BTreeMap<String, Expr<P, R, T>>,

	pub order: Vec<String>,
}

impl<P, R: Ord, T> Match<P, R, T> {
	pub fn map_types<U: Ord>(self, mut f: impl FnMut(T) -> U) -> Match<P, R, U> {
		self.map_types_ref_mut(&mut f)
	}

	pub fn map_types_ref_mut<U: Ord>(self, f: &mut impl FnMut(T) -> U) -> Match<P, R, U> {
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
