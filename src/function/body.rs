use super::BuiltInFunction;
use crate::{
	eval::{Eval, EvalError, Scope},
	expr::Expr,
	rdf::{RdfContext, RdfContextMut},
	Registry, TypeRef, Value,
};
use rdf_types::dataset::{DatasetMut, PatternMatchingDataset};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
		rdf: &impl RdfContext<Resource = R>,
		dataset: &impl PatternMatchingDataset<Resource = R>,
		scope: &Scope<P, R>,
	) -> Result<Value<R>, EvalError<R>> {
		match self {
			Self::Expr(e) => e.eval(registry, rdf, dataset, scope),
			Self::BuiltIn(b) => b.eval(rdf, dataset, scope.top_const_values()),
		}
	}

	pub fn eval_inverse(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &mut impl RdfContextMut<Resource = R>,
		dataset: &mut impl DatasetMut<Resource = R>,
		scope: &mut Scope<P, R>,
		output: &Value<R>,
	) -> Result<(), EvalError<R>> {
		match self {
			Self::Expr(e) => e.eval_inverse(registry, rdf, dataset, scope, output),
			Self::BuiltIn(b) => {
				let offset = scope.parent().unwrap().top_const_values().len() as u32;

				let values = b.eval_inverse(rdf, dataset, scope, output)?;

				for (i, value) in values.into_iter().enumerate() {
					scope.set(rdf, offset + i as u32, value)?;
				}

				Ok(())
			}
		}
	}
}
