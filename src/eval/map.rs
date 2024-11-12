use crate::{expr::Map, Registry, TypeRef, Value};
use rdf_types::dataset::{DatasetMut, PatternMatchingDataset};
use std::collections::BTreeMap;

use super::{Eval, EvalError, EvalTyped, RdfContext, RdfContextMut, Scope};

impl<R: Clone + Ord> EvalTyped<R> for Map<R> {
	fn eval_typed(
		&self,
		registry: &Registry<R>,
		rdf: &impl RdfContext<R>,
		dataset: &impl PatternMatchingDataset<Resource = R>,
		scope: &Scope<R>,
		type_ref: TypeRef,
	) -> Result<Value<R>, EvalError<R>> {
		let type_ = registry.get_type(type_ref).unwrap().as_struct().unwrap();

		let mut result = BTreeMap::new();

		for (key, value) in &self.entries {
			let field = type_.fields.get(&key.type_).unwrap();

			match value.eval(registry, rdf, dataset, scope) {
				Ok(value) => {
					result.insert(key.to_untyped(), value);
				}
				Err(EvalError::MissingData(_)) if !field.required => (),
				Err(e) => return Err(e),
			}
		}

		Ok(Value::Map(result))
	}

	fn eval_inverse_typed(
		&self,
		registry: &Registry<R>,
		rdf: &mut impl RdfContextMut<R>,
		dataset: &mut impl DatasetMut<Resource = R>,
		scope: &mut Scope<R>,
		_type_ref: TypeRef,
		output: &Value<R>,
	) -> Result<(), EvalError<R>> {
		match output {
			Value::Map(map) => {
				for (key, value) in map {
					match self.entries.get_untyped(key) {
						Some(expr) => {
							expr.eval_inverse(registry, rdf, dataset, scope, value)?;
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
