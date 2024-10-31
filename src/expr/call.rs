use educe::Educe;

use crate::{function::FunctionRef, TypeRef};

use super::Expr;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Educe)]
#[educe(PartialOrd(bound = "R: 'static + PartialOrd, T: PartialOrd"))]
#[educe(Ord(bound = "R: 'static + Ord, T: Ord"))]
pub struct Call<R, T = TypeRef> {
	pub function: FunctionRef,
	pub args: Vec<Expr<R, T>>,
}
