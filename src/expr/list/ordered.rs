use std::hash::Hash;

use serde::{Deserialize, Serialize};
use serde_seeded::{DeserializeSeeded, SerializeSeeded};

use crate::{
	expr::{Bindings, Expr},
	rdf::{
		serde::serde_indexed_resource, ResourceBackwardIndex, ResourceForwardIndex, VisitResources,
	},
	TypeRef,
};

/// Select a list of resources and map them.
#[derive(
	Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SerializeSeeded, DeserializeSeeded,
)]
#[seeded(
	ser(seed(ResourceForwardIndex<R>), override_bounds(P: Serialize, R: Eq + Hash, T: Serialize)),
	de(seed(ResourceBackwardIndex<R>), override_bounds(P: Deserialize<'de>, R: Clone + Ord, T: Deserialize<'de>))
)]
pub struct OrderedList<P, R, T = TypeRef<P>> {
	/// List components.
	pub components: ListComponents<R>,

	/// List head expressions.
	pub head: Box<Expr<P, R, T>>,

	/// Bound.
	pub bound: Bindings<P, R, T>,

	/// Expression mapping the items.
	pub body: Box<Expr<P, R, T>>,
}

impl<P, R: Ord, T> OrderedList<P, R, T> {
	pub fn map_types<U: Ord>(self, mut f: impl FnMut(T) -> U) -> OrderedList<P, R, U> {
		self.map_types_ref_mut(&mut f)
	}

	pub fn map_types_ref_mut<U: Ord>(self, f: &mut impl FnMut(T) -> U) -> OrderedList<P, R, U> {
		OrderedList {
			components: self.components,
			head: Box::new(self.head.map_types_ref_mut(f)),
			bound: self.bound.map_types_ref_mut(f),
			body: Box::new(self.body.map_types_ref_mut(f)),
		}
	}
}

impl<P, R, T> VisitResources<R> for OrderedList<P, R, T> {
	fn visit_resources_ref_mut(&self, f: &mut impl FnMut(&R)) {
		self.components.visit_resources_ref_mut(f);
		self.head.visit_resources_ref_mut(f);
		self.bound.visit_resources_ref_mut(f);
		self.body.visit_resources_ref_mut(f)
	}
}

#[derive(
	Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SerializeSeeded, DeserializeSeeded,
)]
#[seeded(
	ser(seed(ResourceForwardIndex<R>), override_bounds(R: Eq + Hash)),
	de(seed(ResourceBackwardIndex<R>), override_bounds(R: Clone + Ord))
)]
pub struct ListComponents<R> {
	#[seeded(with(serde_indexed_resource))]
	pub first: R,

	#[seeded(with(serde_indexed_resource))]
	pub rest: R,

	#[seeded(with(serde_indexed_resource))]
	pub nil: R,
}

impl<R> VisitResources<R> for ListComponents<R> {
	fn visit_resources_ref_mut(&self, f: &mut impl FnMut(&R)) {
		f(&self.first);
		f(&self.rest);
		f(&self.nil)
	}
}
