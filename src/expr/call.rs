use std::sync::Arc;

use educe::Educe;

use crate::{function::FunctionRef, Function, TypeRef};

use super::Expr;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Educe)]
#[educe(PartialOrd(bound = "R: 'static + PartialOrd, T: PartialOrd"))]
#[educe(Ord(bound = "R: 'static + Ord, T: Ord"))]
pub struct Call<R, T = TypeRef<R>> {
	pub function: FunctionRef<R>,
	pub args: Vec<Expr<R, T>>,
}