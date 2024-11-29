use crate::{function::FunctionRef, TypeRef};

use super::Expr;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Call<P, R, T = TypeRef<P>> {
	pub function: FunctionRef<P>,
	pub const_args: Vec<Expr<P, R, T>>,
	pub args: Vec<Expr<P, R, T>>,
}

impl<P, R: Ord, T> Call<P, R, T> {
	pub fn map_types<U: Ord>(self, mut f: impl FnMut(T) -> U) -> Call<P, R, U> {
		self.map_types_ref_mut(&mut f)
	}

	pub fn map_types_ref_mut<U: Ord>(self, f: &mut impl FnMut(T) -> U) -> Call<P, R, U> {
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
