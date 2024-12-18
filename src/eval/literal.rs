use super::{EvalError, EvalTyped, RdfContextMut, Scope};
use crate::{rdf::EvalDataset, Literal, Registry, TypeRef, Value};

impl<P, R: Clone> EvalTyped<P, R> for Literal {
	fn eval_typed(
		&self,
		_registry: &impl Registry<PackageId = P, Resource = R>,
		_rdf: &mut impl RdfContextMut<Resource = R>,
		_dataset: &mut impl EvalDataset<R>,
		_scope: &Scope<P, R>,
		_type_ref: TypeRef<P>,
	) -> Result<Value<R>, EvalError<R>> {
		Ok(Value::Literal(self.clone()))
	}

	fn eval_inverse_typed(
		&self,
		_registry: &impl Registry<PackageId = P, Resource = R>,
		_rdf: &mut impl RdfContextMut<Resource = R>,
		_dataset: &mut impl rdf_types::dataset::DatasetMut<Resource = R>,
		_scope: &mut Scope<P, R>,
		_type_ref: TypeRef<P>,
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
