use crate::{expr::Map, TypeRef, Value};
use rdf_types::dataset::{DatasetMut, PatternMatchingDataset};
use std::collections::BTreeMap;

use super::{Eval, EvalError, EvalTyped, RdfContext, RdfContextMut, ReverseScope, Scope};

impl<R: Clone + Ord> EvalTyped<R> for Map<R> {
	fn eval_typed(
		&self,
		rdf: &impl RdfContext<R>,
		dataset: &impl PatternMatchingDataset<Resource = R>,
		scope: &Scope<R>,
		_type_ref: &TypeRef<R>,
	) -> Result<Value<R>, EvalError<R>> {
		let mut result = BTreeMap::new();

		for (key, value) in &self.entries {
			result.insert(key.to_untyped(), value.eval(rdf, dataset, scope)?);
		}

		Ok(Value::Map(result))
	}

	fn eval_inverse_typed(
		&self,
		rdf: &mut impl RdfContextMut<R>,
		dataset: &mut impl DatasetMut<Resource = R>,
		scope: &mut ReverseScope<R>,
		_type_ref: &TypeRef<R>,
		output: &Value<R>,
	) -> Result<(), EvalError<R>> {
		match output {
			Value::Map(map) => {
				for (key, value) in map {
					match self.entries.get_untyped(key) {
						Some(expr) => {
							expr.eval_inverse(rdf, dataset, scope, value)?;
						}
						None => return Err(EvalError::UnknownKey(key.clone())),
					}
				}

				Ok(())
			}
			_ => Err(EvalError::Ambiguity),
		}
	}
}
