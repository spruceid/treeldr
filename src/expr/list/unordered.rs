use crate::{
	expr::{Bindings, Expr},
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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
