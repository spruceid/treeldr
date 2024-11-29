use super::Expr;
use crate::{value::TypedValueMap, TypeRef};

/// Map expression.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Map<P, R, T = TypeRef<P>> {
	/// Entries.
	pub entries: TypedValueMap<P, R, Expr<P, R, T>, T>,
}

impl<P, R: Ord, T> Map<P, R, T> {
	pub fn map_types<U: Ord>(self, mut f: impl FnMut(T) -> U) -> Map<P, R, U> {
		self.map_types_ref_mut(&mut f)
	}

	pub fn map_types_ref_mut<U: Ord>(self, f: &mut impl FnMut(T) -> U) -> Map<P, R, U> {
		Map {
			entries: self
				.entries
				.into_iter()
				.map(|(ty, e)| (ty.map_types_ref_mut(f), e.map_types_ref_mut(f)))
				.collect(),
		}
	}
}
