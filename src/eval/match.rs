use super::{Eval, EvalError, EvalTyped, RdfContext, RdfContextMut, ReverseScope, Scope};
use crate::{expr::Match, TypeRef, Value};
use rdf_types::dataset::{DatasetMut, PatternMatchingDataset};

impl<R: Clone + Ord> EvalTyped<R> for Match<R> {
	fn eval_typed(
		&self,
		rdf: &impl RdfContext<R>,
		dataset: &impl PatternMatchingDataset<Resource = R>,
		scope: &Scope<R>,
		_type_ref: &TypeRef<R>,
	) -> Result<Value<R>, EvalError<R>> {
		for name in &self.order {
			let expr = self.cases.get(name).unwrap();
			if let Some(scope) = expr.bound.find_one(dataset, scope)? {
				if let Ok(value) = expr.inner.eval_typed(rdf, dataset, &scope, &expr.type_) {
					return Ok(value);
				}
			}
		}

		Err(EvalError::Empty)
	}

	fn eval_inverse_typed(
		&self,
		rdf: &mut impl RdfContextMut<R>,
		dataset: &mut impl DatasetMut<Resource = R>,
		scope: &mut ReverseScope<R>,
		_type_ref: &TypeRef<R>,
		output: &Value<R>,
	) -> Result<(), EvalError<R>> {
		for name in &self.order {
			let expr = self.cases.get(name).unwrap();
			if expr.type_.contains(output) {
				return expr.eval_inverse(rdf, dataset, scope, output);
			}
		}

		Err(EvalError::InvalidType)
	}
}
