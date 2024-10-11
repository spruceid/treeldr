use rdf_types::dataset::{DatasetMut, PatternMatchingDataset};

use crate::{expr::Call, Value};

use super::{Eval, EvalError, RdfContext, RdfContextMut, ReverseScope, Scope};

impl<R: Clone + Ord> Eval<R> for Call<R> {
	fn eval(
		&self,
		rdf: &impl RdfContext<R>,
		dataset: &impl PatternMatchingDataset<Resource = R>,
		scope: &Scope<R>,
	) -> Result<Value<R>, EvalError<R>> {
		let mut evaluated_args = Vec::with_capacity(self.args.len());

		for a in &self.args {
			evaluated_args.push(a.eval(rdf, dataset, scope)?);
		}

		self.function.call(rdf, dataset, &evaluated_args)
	}

	fn eval_inverse(
		&self,
		rdf: &mut impl RdfContextMut<R>,
		dataset: &mut impl DatasetMut<Resource = R>,
		scope: &mut ReverseScope<R>,
		output: &Value<R>,
	) -> Result<(), EvalError<R>> {
		for (arg, value) in self
			.args
			.iter()
			.zip(self.function.call_inverse(rdf, dataset, output)?)
		{
			arg.eval_inverse(rdf, dataset, scope, &value)?;
		}

		Ok(())
	}
}
