mod bound;
pub use bound::*;

mod list;
use educe::Educe;
pub use list::*;

mod map;
pub use map::*;

mod r#match;
pub use r#match::*;

mod call;
pub use call::*;

use crate::{Domain, Literal, TypeRef};

/// Expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Educe)]
#[educe(PartialOrd(bound = "R: 'static + PartialOrd, T: PartialOrd"))]
#[educe(Ord(bound = "R: 'static + Ord, T: Ord"))]
pub struct Expr<R, T = TypeRef<R>> {
	pub type_: T,

	pub bound: Bound<R, T>,

	pub inner: InnerExpr<R, T>,
}

/// Inner expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Educe)]
#[educe(PartialOrd(bound = "R: 'static + PartialOrd, T: PartialOrd"))]
#[educe(Ord(bound = "R: 'static + Ord, T: Ord"))]
pub enum InnerExpr<R, T = TypeRef<R>> {
	/// Resource variable.
	///
	/// ```tldr
	/// ?variable
	/// ```
	Var(u32),

	/// Literal value.
	///
	/// ```tldr
	/// "value"
	/// ```
	Literal(Literal),

	/// List.
	///
	/// ```tldr
	/// [ value1, ..., valueN ]
	/// [ all ?x where { ... } expr ]
	/// [ list ?x in ?list where { ... } expr ]
	/// ```
	List(List<R, T>),

	/// Map.
	///
	/// ```tldr
	/// { key1 = value1, ..., keyN = valueN }
	/// ```
	Map(Map<R, T>),

	/// Match.
	Match(Match<R, T>),

	/// Function call.
	Call(Call<R, T>),
}

impl<R, T> InnerExpr<R, T> {
	pub fn as_map(&self) -> Option<&Map<R, T>> {
		match self {
			Self::Map(r) => Some(r),
			_ => None,
		}
	}
}

impl<R, T> Domain for InnerExpr<R, T> {
	type Resource = R;
}
