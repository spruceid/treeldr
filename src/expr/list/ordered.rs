use crate::{
	expr::{Bindings, Expr},
	TypeRef,
};

/// Select a list of resources and map them.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OrderedList<R, T = TypeRef> {
	/// List components.
	pub components: ListComponents<R>,

	/// List head expressions.
	pub head: Box<Expr<R, T>>,

	/// Bound.
	pub bound: Bindings<R, T>,

	/// Expression mapping the items.
	pub body: Box<Expr<R, T>>,
}

impl<R: Ord, T> OrderedList<R, T> {
	pub fn map_types<U: Ord>(self, mut f: impl FnMut(T) -> U) -> OrderedList<R, U> {
		self.map_types_ref_mut(&mut f)
	}

	pub fn map_types_ref_mut<U: Ord>(self, f: &mut impl FnMut(T) -> U) -> OrderedList<R, U> {
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
