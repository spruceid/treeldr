mod ordered;
use educe::Educe;
pub use ordered::*;

mod unordered;
pub use unordered::*;

use crate::TypeRef;

use super::Expr;

/// List expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Educe)]
#[educe(PartialOrd(bound = "R: 'static + PartialOrd, T: PartialOrd"))]
#[educe(Ord(bound = "R: 'static + Ord, T: Ord"))]
pub enum List<R, T = TypeRef<R>> {
	Explicit(ExplicitList<R, T>),
	Implicit(ImplicitList<R, T>),
}

/// Explicit list (tuple construction).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Educe)]
#[educe(PartialOrd(bound = "R: 'static + PartialOrd, T: PartialOrd"))]
#[educe(Ord(bound = "R: 'static + Ord, T: Ord"))]
pub struct ExplicitList<R, T = TypeRef<R>> {
	pub items: Vec<Expr<R, T>>,
}

impl<R, T> ExplicitList<R, T> {
	pub fn new(items: Vec<Expr<R, T>>) -> Self {
		Self {
			items
		}
	}
}

/// Implicit list expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Educe)]
#[educe(PartialOrd(bound = "R: 'static + PartialOrd, T: PartialOrd"))]
#[educe(Ord(bound = "R: 'static + Ord, T: Ord"))]
pub enum ImplicitList<R, T = TypeRef<R>> {
	Ordered(OrderedList<R, T>),
	Unordered(UnorderedList<R, T>),
}
