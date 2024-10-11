use crate::{Literal, TypeRef, Value};

use super::{EvalError, EvalTyped, RdfContext, RdfContextMut, ReverseScope, Scope};

impl<R: Clone> EvalTyped<R> for Literal {
	fn eval_typed(
		&self,
		_rdf: &impl RdfContext<R>,
		_dataset: &impl rdf_types::dataset::PatternMatchingDataset<Resource = R>,
		_scope: &Scope<R>,
		_type_ref: &TypeRef<R>,
	) -> Result<Value<R>, EvalError<R>> {
		Ok(Value::Literal(self.clone()))
	}

	fn eval_inverse_typed(
		&self,
		_rdf: &mut impl RdfContextMut<R>,
		_dataset: &mut impl rdf_types::dataset::DatasetMut<Resource = R>,
		_scope: &mut ReverseScope<R>,
		_type_ref: &TypeRef<R>,
		output: &Value<R>,
	) -> Result<(), EvalError<R>> {
		match output {
			Value::Literal(l) => {
				if l != self {
					return Err(EvalError::InvalidValue);
				}

				Ok(())
			}
			_ => Err(EvalError::InvalidType),
		}
	}
}
