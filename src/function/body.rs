use std::hash::Hash;

use super::BuiltInFunction;
use crate::{
	eval::{Eval, EvalContext, EvalError, Scope},
	expr::Expr,
	rdf::{ResourceBackwardIndex, ResourceForwardIndex, VisitResources},
	Registry, TypeRef, Value,
};
use serde::{Deserialize, Serialize};
use serde_seeded::{DeserializeSeeded, SerializeSeeded};

#[derive(
	Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SerializeSeeded, DeserializeSeeded,
)]
#[seeded(
	ser(seed(ResourceForwardIndex<R>), override_bounds(P: Serialize, R: Eq + Hash, T: Serialize)),
	de(seed(ResourceBackwardIndex<R>), override_bounds(P: Deserialize<'de>, R: Clone + Ord, T: Deserialize<'de>), bounds(T: Ord))
)]
pub enum Body<P, R, T = TypeRef<P>> {
	BuiltIn(BuiltInFunction),
	Expr(Box<Expr<P, R, T>>),
}

impl<P, R: Ord, T> Body<P, R, T> {
	pub fn map_types<U: Ord>(self, f: impl FnMut(T) -> U) -> Body<P, R, U> {
		match self {
			Self::BuiltIn(b) => Body::BuiltIn(b),
			Self::Expr(e) => Body::Expr(Box::new(e.map_types(f))),
		}
	}
}

impl<P: Copy + Ord, R: Ord + Clone> Body<P, R> {
	pub fn eval(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &impl EvalContext<Resource = R>,
		scope: &Scope<P, R>,
	) -> Result<Value<R>, EvalError<R>> {
		match self {
			Self::Expr(e) => e.eval(registry, rdf, scope),
			Self::BuiltIn(b) => b.eval(rdf, scope.top_const_values()),
		}
	}

	pub fn eval_inverse(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &mut impl EvalContext<Resource = R>,
		scope: &mut Scope<P, R>,
		output: &Value<R>,
	) -> Result<(), EvalError<R>> {
		match self {
			Self::Expr(e) => e.eval_inverse(registry, rdf, scope, output),
			Self::BuiltIn(b) => {
				// The const arguments are in the parent scope.
				let offset = scope.parent().unwrap().top_const_values().len() as u32;

				let values = b.eval_inverse(rdf, scope, output)?;

				for (i, value) in values.into_iter().enumerate() {
					scope.set(rdf, offset + i as u32, value)?;
				}

				Ok(())
			}
		}
	}
}

impl<P, R, T> VisitResources<R> for Body<P, R, T> {
	fn visit_resources_ref_mut(&self, f: &mut impl FnMut(&R)) {
		match self {
			Self::BuiltIn(_) => (),
			Self::Expr(e) => e.visit_resources_ref_mut(f),
		}
	}
}
