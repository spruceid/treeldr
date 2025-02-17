use super::{Eval, EvalContext, EvalError, EvalTyped, MissingData, Scope};
use crate::{expr::Match, Registry, TypeRef, Value};

impl<P: Copy + Ord, R: Clone + Ord> EvalTyped<P, R> for Match<P, R> {
	fn eval_typed(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &impl EvalContext<Resource = R>,
		scope: &Scope<P, R>,
		_type_ref: TypeRef<P>,
	) -> Result<Value<R>, EvalError<R>> {
		for name in &self.order {
			let expr = self.cases.get(name).unwrap();
			match expr.eval(registry, rdf, scope) {
				Ok(value) => return Ok(value),
				Err(EvalError::MissingData(_)) => (),
				Err(e) => return Err(e),
			}
		}

		Err(EvalError::MissingData(MissingData::NotVariantMatch))
	}

	fn eval_inverse_typed(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &mut impl EvalContext<Resource = R>,
		scope: &mut Scope<P, R>,
		_type_ref: TypeRef<P>,
		output: &Value<R>,
	) -> Result<(), EvalError<R>> {
		for name in &self.order {
			let expr = self.cases.get(name).unwrap();
			if expr.r#type.contains(registry, output) {
				return expr.eval_inverse(registry, rdf, scope, output);
			}
		}

		Err(EvalError::InvalidType)
	}
}
