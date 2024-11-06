use rdf_types::dataset::{DatasetMut, PatternMatchingDataset};

use crate::{
	eval::{Eval, EvalError, ReverseScope, Scope}, expr::Expr, rdf::{RdfContext, RdfContextMut}, Registry, TypeRef, Value
};

use super::BuiltInFunction;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Body<R, T = TypeRef> {
	BuiltIn(BuiltInFunction),
	Expr(Box<Expr<R, T>>),
}

impl<R: Ord, T> Body<R, T> {
	pub fn map_types<U: Ord>(self, f: impl FnMut(T) -> U) -> Body<R, U> {
		match self {
			Self::BuiltIn(b) => Body::BuiltIn(b),
			Self::Expr(e) => Body::Expr(Box::new(e.map_types(f)))
		}
	}
}

impl<R: Ord + Clone> Body<R> {
	pub fn eval(
		&self,
		registry: &Registry<R>,
		rdf: &impl RdfContext<R>,
		dataset: &impl PatternMatchingDataset<Resource = R>,
		scope: &Scope<R>,
	) -> Result<Value<R>, EvalError<R>> {
		match self {
			Self::Expr(e) => e.eval(registry, rdf, dataset, scope),
			Self::BuiltIn(b) => b.eval(dataset, scope),
		}
	}

	pub fn eval_inverse(
		&self,
		registry: &Registry<R>,
		rdf: &mut impl RdfContextMut<R>,
		dataset: &mut impl DatasetMut<Resource = R>,
		input_types: &[TypeRef],
		output: &Value<R>,
	) -> Result<Vec<Value<R>>, EvalError<R>> {
		match self {
			Self::Expr(e) => {
				let mut scope = ReverseScope::new(None, input_types);
				e.eval_inverse(registry, rdf, dataset, &mut scope, output)?;
				scope.end(rdf)
			}
			Self::BuiltIn(b) => b.eval_inverse(dataset, output),
		}
	}
}
