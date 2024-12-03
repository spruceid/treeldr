use std::hash::Hash;

use serde::{Deserialize, Serialize};
use serde_seeded::{DeserializeSeeded, SerializeSeeded};

use crate::{
	function::FunctionRef,
	rdf::{ResourceBackwardIndex, ResourceForwardIndex, VisitResources},
	TypeRef,
};

use super::Expr;

#[derive(
	Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SerializeSeeded, DeserializeSeeded,
)]
#[seeded(
	ser(seed(ResourceForwardIndex<R>), override_bounds(P: Serialize, R: Eq + Hash, T: Serialize)),
	de(seed(ResourceBackwardIndex<R>), override_bounds(P: Deserialize<'de>, R: Clone + Ord, T: Deserialize<'de>))
)]
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

impl<P, R, T> VisitResources<R> for Call<P, R, T> {
	fn visit_resources_ref_mut(&self, f: &mut impl FnMut(&R)) {
		self.const_args.visit_resources_ref_mut(f);
		self.args.visit_resources_ref_mut(f)
	}
}
