use crate::{function::FunctionRef, TypeRef};

use super::Expr;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Call<R, T = TypeRef> {
	pub function: FunctionRef,
	pub const_args: Vec<Expr<R, T>>,
	pub args: Vec<Expr<R, T>>,
}

impl<R: Ord, T> Call<R, T> {
	pub fn map_types<U: Ord>(self, mut f: impl FnMut(T) -> U) -> Call<R, U> {
		self.map_types_ref_mut(&mut f)
	}

	pub fn map_types_ref_mut<U: Ord>(self, f: &mut impl FnMut(T) -> U) -> Call<R, U> {
		Call {
			function: self.function,
			const_args: self
				.const_args
				.into_iter()
				.map(|v| v.map_types_ref_mut(f))
				.collect(),
			args: self
				.args
				.into_iter()
				.map(|e| e.map_types_ref_mut(f))
				.collect(),
		}
	}
}
