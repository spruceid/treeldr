use educe::Educe;

use super::Expr;
use crate::{value::TypedMap, TypeRef};

/// Map expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Educe)]
#[educe(PartialOrd(bound = "R: 'static + PartialOrd, T: PartialOrd"))]
#[educe(Ord(bound = "R: 'static + Ord, T: Ord"))]
pub struct Map<R, T = TypeRef<R>> {
	/// Entries.
	pub entries: TypedMap<R, Expr<R, T>, T>,
}
