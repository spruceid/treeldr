use educe::Educe;

use crate::{expr::{Bound, Expr}, TypeRef};

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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Educe)]
#[educe(PartialOrd(bound = "R: 'static + PartialOrd, T: PartialOrd"))]
#[educe(Ord(bound = "R: 'static + Ord, T: Ord"))]
pub struct UnorderedList<R, T = TypeRef<R>> {
	pub bound: Bound<R, T>,

	/// Expression mapping the selected resources.
	pub body: Box<Expr<R, T>>,
}
