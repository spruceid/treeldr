use super::Expr;
use crate::{value::TypedMap, TypeRef};

/// Map expression.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Map<R, T = TypeRef> {
	/// Entries.
	pub entries: TypedMap<R, Expr<R, T>, T>,
}

impl<R: Ord, T> Map<R, T> {
	pub fn map_types<U: Ord>(self, mut f: impl FnMut(T) -> U) -> Map<R, U> {
		self.map_types_ref_mut(&mut f)
	}

	pub fn map_types_ref_mut<U: Ord>(self, f: &mut impl FnMut(T) -> U) -> Map<R, U> {
		Map {
			entries: self.entries.into_iter().map(|(ty, e)| {
				(
					ty.map_types_ref_mut(f),
					e.map_types_ref_mut(f)
				)
			}).collect()
		}
	}
}