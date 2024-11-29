mod bound;
pub use bound::*;

mod list;
pub use list::*;

mod map;
pub use map::*;

mod r#match;
pub use r#match::*;

mod call;
pub use call::*;

use crate::{Literal, TypeRef};

/// Expression.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Expr<P, R, T = TypeRef<P>> {
	pub type_: T,

	pub inner: InnerExpr<P, R, T>,
}

impl<P, R: Ord, T> Expr<P, R, T> {
	pub fn map_types<U: Ord>(self, mut f: impl FnMut(T) -> U) -> Expr<P, R, U> {
		self.map_types_ref_mut(&mut f)
	}

	pub fn map_types_ref_mut<U: Ord>(self, f: &mut impl FnMut(T) -> U) -> Expr<P, R, U> {
		Expr {
			type_: f(self.type_),
			inner: self.inner.map_types_ref_mut(f),
		}
	}
}

/// Inner expression.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InnerExpr<P, R, T = TypeRef<P>> {
	/// Let binding.
	Let(Bindings<P, R, T>, Box<Expr<P, R, T>>),

	/// Resource variable.
	///
	/// ```tldr
	/// ?variable
	/// ```
	Var(u32),

	Resource(R),

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
	List(List<P, R, T>),

	/// Map.
	///
	/// ```tldr
	/// { key1 = value1, ..., keyN = valueN }
	/// ```
	Map(Map<P, R, T>),

	/// Match.
	Match(Match<P, R, T>),

	/// Function call.
	Call(Call<P, R, T>),
}

impl<P, R, T> InnerExpr<P, R, T> {
	pub fn as_map(&self) -> Option<&Map<P, R, T>> {
		match self {
			Self::Map(r) => Some(r),
			_ => None,
		}
	}
}

impl<P, R: Ord, T> InnerExpr<P, R, T> {
	pub fn map_types<U: Ord>(self, mut f: impl FnMut(T) -> U) -> InnerExpr<P, R, U> {
		self.map_types_ref_mut(&mut f)
	}

	fn map_types_ref_mut<U: Ord>(self, f: &mut impl FnMut(T) -> U) -> InnerExpr<P, R, U> {
		match self {
			Self::Let(b, e) => {
				InnerExpr::Let(b.map_types_ref_mut(f), Box::new(e.map_types_ref_mut(f)))
			}
			Self::Var(x) => InnerExpr::Var(x),
			Self::Resource(r) => InnerExpr::Resource(r),
			Self::Literal(l) => InnerExpr::Literal(l),
			Self::List(l) => InnerExpr::List(l.map_types_ref_mut(f)),
			Self::Map(m) => InnerExpr::Map(m.map_types_ref_mut(f)),
			Self::Match(m) => InnerExpr::Match(m.map_types_ref_mut(f)),
			Self::Call(m) => InnerExpr::Call(m.map_types_ref_mut(f)),
		}
	}
}
