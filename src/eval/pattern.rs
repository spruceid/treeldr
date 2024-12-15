use super::{Eval, EvalError, Scope};
use crate::{
	pattern::{CallPattern, PatternDesc},
	rdf::RdfContextMut,
	Pattern, Registry, Value,
};
use rdf_types::dataset::{DatasetMut, PatternMatchingDataset};

impl<P: Copy + Ord, R: Clone + Ord> Pattern<P, R> {
	pub fn eval<'s>(
		&'s self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &mut impl RdfContextMut<Resource = R>,
		dataset: &mut (impl DatasetMut<Resource = R> + PatternMatchingDataset),
		scope: &'s Scope<'s, P, R>,
		value: Value<R>,
	) -> Result<Vec<Value<R>>, EvalError<R>> {
		let mut result = Vec::new();
		self.desc
			.eval(registry, rdf, dataset, scope, &mut result, value)?;
		Ok(result)
	}

	pub fn eval_inverse<'s>(
		&'s self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &mut impl RdfContextMut<Resource = R>,
		dataset: &mut (impl DatasetMut<Resource = R> + PatternMatchingDataset),
		scope: &'s Scope<'s, P, R>,
		values: &mut Vec<Value<R>>,
	) -> Result<Value<R>, EvalError<R>> {
		self.desc
			.eval_inverse(registry, rdf, dataset, scope, values)
	}
}

impl<P: Copy + Ord, R: Clone + Ord> PatternDesc<P, R> {
	pub fn eval<'s>(
		&'s self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &mut impl RdfContextMut<Resource = R>,
		dataset: &mut (impl DatasetMut<Resource = R> + PatternMatchingDataset),
		scope: &'s Scope<'s, P, R>,
		result: &mut Vec<Value<R>>,
		value: Value<R>,
	) -> Result<(), EvalError<R>> {
		match (self, value) {
			(Self::Var(_), value) => {
				result.push(value);
				Ok(())
			}
			(Self::List(_), Value::List(_)) => {
				todo!()
			}
			(Self::Map(_), Value::Map(_)) => {
				todo!()
			}
			(Self::Call(call), value) => call.eval(registry, rdf, dataset, scope, result, value),
			_ => {
				todo!()
			}
		}
	}

	pub fn eval_inverse<'s>(
		&'s self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &mut impl RdfContextMut<Resource = R>,
		dataset: &mut (impl DatasetMut<Resource = R> + PatternMatchingDataset),
		scope: &'s Scope<'s, P, R>,
		values: &mut Vec<Value<R>>,
	) -> Result<Value<R>, EvalError<R>> {
		match self {
			Self::Var(_) => Ok(values.pop().unwrap()),
			Self::List(_) => {
				todo!()
			}
			Self::Map(_) => {
				todo!()
			}
			Self::Call(call) => call.eval_inverse(registry, rdf, dataset, scope, values),
		}
	}
}

impl<P: Copy + Ord, R: Clone + Ord> CallPattern<P, R> {
	pub fn eval<'s>(
		&'s self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &mut impl RdfContextMut<Resource = R>,
		dataset: &mut (impl DatasetMut<Resource = R> + PatternMatchingDataset),
		scope: &'s Scope<'s, P, R>,
		result: &mut Vec<Value<R>>,
		output: Value<R>,
	) -> Result<(), EvalError<R>> {
		let const_args: Vec<_> = self
			.const_args
			.iter()
			.map(|a| a.eval(registry, rdf, dataset, scope))
			.collect::<Result<_, _>>()?;

		let values = registry.get_function(self.function).unwrap().call_inverse(
			registry,
			rdf,
			dataset,
			&const_args,
			&output,
		)?;

		for (arg, value) in self.args.iter().zip(values) {
			arg.desc
				.eval(registry, rdf, dataset, scope, result, value)?;
		}

		Ok(())
	}

	pub fn eval_inverse<'s>(
		&'s self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &mut impl RdfContextMut<Resource = R>,
		dataset: &mut (impl DatasetMut<Resource = R> + PatternMatchingDataset),
		scope: &'s Scope<'s, P, R>,
		values: &mut Vec<Value<R>>,
	) -> Result<Value<R>, EvalError<R>> {
		let mut evaluated_args = Vec::with_capacity(self.const_args.len() + self.args.len());

		for a in &self.const_args {
			evaluated_args.push(a.eval(registry, rdf, dataset, scope)?);
		}

		for a in &self.args {
			evaluated_args.push(a.eval_inverse(registry, rdf, dataset, scope, values)?);
		}

		registry
			.get_function(self.function)
			.unwrap()
			.call(registry, rdf, dataset, &evaluated_args)
	}
}
