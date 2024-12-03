mod ordered;
use std::hash::Hash;

pub use ordered::*;

mod unordered;
use serde::{Deserialize, Serialize};
use serde_seeded::{DeserializeSeeded, SerializeSeeded};
pub use unordered::*;

use crate::{
	rdf::{ResourceBackwardIndex, ResourceForwardIndex, VisitResources},
	TypeRef,
};

use super::Expr;

/// List expression.
#[derive(
	Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SerializeSeeded, DeserializeSeeded,
)]
#[seeded(
	ser(seed(ResourceForwardIndex<R>), override_bounds(P: Serialize, R: Eq + Hash, T: Serialize)),
	de(seed(ResourceBackwardIndex<R>), override_bounds(P: Deserialize<'de>, R: Clone + Ord, T: Deserialize<'de>))
)]
pub enum List<P, R, T = TypeRef<P>> {
	Explicit(ExplicitList<P, R, T>),
	Implicit(ImplicitList<P, R, T>),
}

impl<P, R: Ord, T> List<P, R, T> {
	pub fn map_types<U: Ord>(self, mut f: impl FnMut(T) -> U) -> List<P, R, U> {
		self.map_types_ref_mut(&mut f)
	}

	pub fn map_types_ref_mut<U: Ord>(self, f: &mut impl FnMut(T) -> U) -> List<P, R, U> {
		match self {
			Self::Explicit(e) => List::Explicit(e.map_types_ref_mut(f)),
			Self::Implicit(i) => List::Implicit(i.map_types_ref_mut(f)),
		}
	}
}

impl<P, R, T> VisitResources<R> for List<P, R, T> {
	fn visit_resources_ref_mut(&self, f: &mut impl FnMut(&R)) {
		match self {
			Self::Explicit(e) => e.visit_resources_ref_mut(f),
			Self::Implicit(i) => i.visit_resources_ref_mut(f),
		}
	}
}

/// Explicit list (tuple construction).
#[derive(
	Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SerializeSeeded, DeserializeSeeded,
)]
#[seeded(
	ser(seed(ResourceForwardIndex<R>), override_bounds(P: Serialize, R: Eq + Hash, T: Serialize)),
	de(seed(ResourceBackwardIndex<R>), override_bounds(P: Deserialize<'de>, R: Clone + Ord, T: Deserialize<'de>))
)]
pub struct ExplicitList<P, R, T = TypeRef<P>> {
	pub items: Vec<Expr<P, R, T>>,
}

impl<P, R, T> ExplicitList<P, R, T> {
	pub fn new(items: Vec<Expr<P, R, T>>) -> Self {
		Self { items }
	}
}

impl<P, R: Ord, T> ExplicitList<P, R, T> {
	pub fn map_types<U: Ord>(self, mut f: impl FnMut(T) -> U) -> ExplicitList<P, R, U> {
		self.map_types_ref_mut(&mut f)
	}

	fn map_types_ref_mut<U: Ord>(self, f: &mut impl FnMut(T) -> U) -> ExplicitList<P, R, U> {
		ExplicitList {
			items: self
				.items
				.into_iter()
				.map(|e| e.map_types_ref_mut(f))
				.collect(),
		}
	}
}

impl<P, R, T> VisitResources<R> for ExplicitList<P, R, T> {
	fn visit_resources_ref_mut(&self, f: &mut impl FnMut(&R)) {
		self.items.visit_resources_ref_mut(f)
	}
}

/// Implicit list expression.
#[derive(
	Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SerializeSeeded, DeserializeSeeded,
)]
#[seeded(
	ser(seed(ResourceForwardIndex<R>), override_bounds(P: Serialize, R: Eq + Hash, T: Serialize)),
	de(seed(ResourceBackwardIndex<R>), override_bounds(P: Deserialize<'de>, R: Clone + Ord, T: Deserialize<'de>))
)]
pub enum ImplicitList<P, R, T = TypeRef<P>> {
	Ordered(OrderedList<P, R, T>),
	Unordered(UnorderedList<P, R, T>),
}

impl<P, R: Ord, T> ImplicitList<P, R, T> {
	pub fn map_types<U: Ord>(self, mut f: impl FnMut(T) -> U) -> ImplicitList<P, R, U> {
		self.map_types_ref_mut(&mut f)
	}

	fn map_types_ref_mut<U: Ord>(self, f: &mut impl FnMut(T) -> U) -> ImplicitList<P, R, U> {
		match self {
			Self::Ordered(o) => ImplicitList::Ordered(o.map_types_ref_mut(f)),
			Self::Unordered(u) => ImplicitList::Unordered(u.map_types_ref_mut(f)),
		}
	}
}

impl<P, R, T> VisitResources<R> for ImplicitList<P, R, T> {
	fn visit_resources_ref_mut(&self, f: &mut impl FnMut(&R)) {
		match self {
			Self::Ordered(o) => o.visit_resources_ref_mut(f),
			Self::Unordered(u) => u.visit_resources_ref_mut(f),
		}
	}
}
