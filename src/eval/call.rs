use rdf_types::dataset::{DatasetMut, PatternMatchingDataset};

use crate::{expr::Call, Registry, Value};

use super::{Eval, EvalError, RdfContext, RdfContextMut, ReverseScope, Scope};

impl<R: Clone + Ord> Eval<R> for Call<R> {
	fn eval(
		&self,
		registry: &Registry<R>,
		rdf: &impl RdfContext<R>,
		dataset: &impl PatternMatchingDataset<Resource = R>,
		scope: &Scope<R>,
	) -> Result<Value<R>, EvalError<R>> {
		let mut evaluated_args = Vec::with_capacity(self.args.len());

		for a in &self.args {
			evaluated_args.push(a.eval(registry, rdf, dataset, scope)?);
		}

		registry
			.get_function(self.function)
			.unwrap()
			.call(registry, rdf, dataset, &evaluated_args)
	}

	fn eval_inverse(
		&self,
		registry: &Registry<R>,
		rdf: &mut impl RdfContextMut<R>,
		dataset: &mut impl DatasetMut<Resource = R>,
		scope: &mut ReverseScope<R>,
		output: &Value<R>,
	) -> Result<(), EvalError<R>> {
		let values = registry
			.get_function(self.function)
			.unwrap()
			.call_inverse(registry, rdf, dataset, output)?;

		for (arg, value) in self.args.iter().zip(values) {
			arg.eval_inverse(registry, rdf, dataset, scope, &value)?;
		}

		Ok(())
	}
}
