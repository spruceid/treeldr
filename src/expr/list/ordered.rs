use educe::Educe;

use crate::{expr::{Bound, Expr}, TypeRef};

/// Select a list of resources and map them.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Educe)]
#[educe(PartialOrd(bound = "R: 'static + PartialOrd, T: PartialOrd"))]
#[educe(Ord(bound = "R: 'static + Ord, T: Ord"))]
pub struct OrderedList<R, T = TypeRef<R>> {
	/// List components.
	pub components: ListComponents<R>,

	/// List head expressions.
	pub head: Box<Expr<R, T>>,

	/// Bound.
	pub bound: Bound<R, T>,

	/// Expression mapping the items.
	pub body: Box<Expr<R, T>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Educe)]
#[educe(PartialOrd(bound = "R: 'static + PartialOrd"))]
#[educe(Ord(bound = "R: 'static + Ord"))]
pub struct ListComponents<R> {
	pub first: R,
	pub rest: R,
	pub nil: R,
}