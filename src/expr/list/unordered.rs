use std::hash::Hash;

use serde::{Deserialize, Serialize};
use serde_seeded::{DeserializeSeeded, SerializeSeeded};

use crate::{
	expr::{Bindings, Expr},
	rdf::{ResourceBackwardIndex, ResourceForwardIndex, VisitResources},
	TypeRef,
};

/// Select a set of resources and map them.
///
/// ```abnf
/// select-set = "[" "all" ident-list where-bound expr "]"
/// ```
///
/// # Example
/// ```tldr
/// forall ?self. { cities = [ all ?city where { ?self ex:city ?city } (string_of ?city) ] }
/// ```
#[derive(
	Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SerializeSeeded, DeserializeSeeded,
)]
#[seeded(
	ser(seed(ResourceForwardIndex<R>), override_bounds(P: Serialize, R: Eq + Hash, T: Serialize)),
	de(seed(ResourceBackwardIndex<R>), override_bounds(P: Deserialize<'de>, R: Clone + Ord, T: Deserialize<'de>))
)]
pub struct UnorderedList<P, R, T = TypeRef<P>> {
	pub bound: Bindings<P, R, T>,

	/// Expression mapping the selected resources.
	pub body: Box<Expr<P, R, T>>,
}

impl<P, R: Ord, T> UnorderedList<P, R, T> {
	pub fn map_types<U: Ord>(self, mut f: impl FnMut(T) -> U) -> UnorderedList<P, R, U> {
		self.map_types_ref_mut(&mut f)
	}

	pub fn map_types_ref_mut<U: Ord>(self, f: &mut impl FnMut(T) -> U) -> UnorderedList<P, R, U> {
		UnorderedList {
			bound: self.bound.map_types_ref_mut(f),
			body: Box::new(self.body.map_types_ref_mut(f)),
		}
	}
}

impl<P, R, T> VisitResources<R> for UnorderedList<P, R, T> {
	fn visit_resources_ref_mut(&self, f: &mut impl FnMut(&R)) {
		self.bound.visit_resources_ref_mut(f);
		self.body.visit_resources_ref_mut(f)
	}
}
