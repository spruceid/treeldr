use std::hash::Hash;

use serde::{Deserialize, Serialize};
use serde_seeded::{DeserializeSeeded, SerializeSeeded};

use super::Expr;
use crate::{
	rdf::{ResourceBackwardIndex, ResourceForwardIndex, VisitResources},
	value::TypedValueMap,
	TypeRef,
};

/// Map expression.
#[derive(
	Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SerializeSeeded, DeserializeSeeded,
)]
#[seeded(
	ser(seed(ResourceForwardIndex<R>), override_bounds(P: Serialize, R: Eq + Hash, T: Serialize)),
	de(seed(ResourceBackwardIndex<R>), override_bounds(P: Deserialize<'de>, R: Clone + Ord, T: Deserialize<'de>))
)]
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

impl<P, R, T> VisitResources<R> for Map<P, R, T> {
	fn visit_resources_ref_mut(&self, f: &mut impl FnMut(&R)) {
		self.entries.visit_resources_ref_mut(f)
	}
}
