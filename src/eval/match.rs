use super::{Eval, EvalError, EvalTyped, MissingData, RdfContextMut, Scope};
use crate::{expr::Match, Registry, TypeRef, Value};
use rdf_types::dataset::{DatasetMut, PatternMatchingDataset};

impl<P: Copy + Ord, R: Clone + Ord> EvalTyped<P, R> for Match<P, R> {
	fn eval_typed(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &mut impl RdfContextMut<Resource = R>,
		dataset: &mut (impl DatasetMut<Resource = R> + PatternMatchingDataset),
		scope: &Scope<P, R>,
		_type_ref: TypeRef<P>,
	) -> Result<Value<R>, EvalError<R>> {
		for name in &self.order {
			let expr = self.cases.get(name).unwrap();
			match expr.eval(registry, rdf, dataset, scope) {
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
		rdf: &mut impl RdfContextMut<Resource = R>,
		dataset: &mut (impl DatasetMut<Resource = R> + PatternMatchingDataset),
		scope: &mut Scope<P, R>,
		_type_ref: TypeRef<P>,
		output: &Value<R>,
	) -> Result<(), EvalError<R>> {
		for name in &self.order {
			let expr = self.cases.get(name).unwrap();
			if expr.r#type.contains(registry, output) {
				return expr.eval_inverse(registry, rdf, dataset, scope, output);
			}
		}

		Err(EvalError::InvalidType)
	}
}
