use educe::Educe;
use std::collections::BTreeMap;

use crate::TypeRef;

use super::Expr;

/// Match expressions.
///
/// ```tldr
/// match {
/// 	foo: ?x where { ... } {
/// 		// ...
/// 	},
/// 	bar: ?y where { ... } {
/// 		// ...
/// 	}
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Educe)]
#[educe(PartialOrd(bound = "R: 'static + PartialOrd, T: PartialOrd"))]
#[educe(Ord(bound = "R: 'static + Ord, T: Ord"))]
pub struct Match<R, T = TypeRef<R>> {
	pub cases: BTreeMap<String, Expr<R, T>>,

	pub order: Vec<String>,
}
