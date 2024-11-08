mod ordered;
pub use ordered::*;

mod unordered;
pub use unordered::*;

use crate::TypeRef;

use super::Expr;

/// List expression.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum List<R, T = TypeRef> {
	Explicit(ExplicitList<R, T>),
	Implicit(ImplicitList<R, T>),
}

impl<R: Ord, T> List<R, T> {
	pub fn map_types<U: Ord>(self, mut f: impl FnMut(T) -> U) -> List<R, U> {
		self.map_types_ref_mut(&mut f)
	}

	pub fn map_types_ref_mut<U: Ord>(self, f: &mut impl FnMut(T) -> U) -> List<R, U> {
		match self {
			Self::Explicit(e) => List::Explicit(e.map_types_ref_mut(f)),
			Self::Implicit(i) => List::Implicit(i.map_types_ref_mut(f)),
		}
	}
}

/// Explicit list (tuple construction).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExplicitList<R, T = TypeRef> {
	pub items: Vec<Expr<R, T>>,
}

impl<R, T> ExplicitList<R, T> {
	pub fn new(items: Vec<Expr<R, T>>) -> Self {
		Self { items }
	}
}

impl<R: Ord, T> ExplicitList<R, T> {
	pub fn map_types<U: Ord>(self, mut f: impl FnMut(T) -> U) -> ExplicitList<R, U> {
		self.map_types_ref_mut(&mut f)
	}

	fn map_types_ref_mut<U: Ord>(self, f: &mut impl FnMut(T) -> U) -> ExplicitList<R, U> {
		ExplicitList {
			items: self
				.items
				.into_iter()
				.map(|e| e.map_types_ref_mut(f))
				.collect(),
		}
	}
}

/// Implicit list expression.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImplicitList<R, T = TypeRef> {
	Ordered(OrderedList<R, T>),
	Unordered(UnorderedList<R, T>),
}

impl<R: Ord, T> ImplicitList<R, T> {
	pub fn map_types<U: Ord>(self, mut f: impl FnMut(T) -> U) -> ImplicitList<R, U> {
		self.map_types_ref_mut(&mut f)
	}

	fn map_types_ref_mut<U: Ord>(self, f: &mut impl FnMut(T) -> U) -> ImplicitList<R, U> {
		match self {
			Self::Ordered(o) => ImplicitList::Ordered(o.map_types_ref_mut(f)),
			Self::Unordered(u) => ImplicitList::Unordered(u.map_types_ref_mut(f)),
		}
	}
}
