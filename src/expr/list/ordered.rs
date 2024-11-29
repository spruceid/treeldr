use crate::{
	expr::{Bindings, Expr},
	TypeRef,
};

/// Select a list of resources and map them.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ListComponents<R> {
	pub first: R,
	pub rest: R,
	pub nil: R,
}
