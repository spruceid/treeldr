use super::{Eval, EvalError, Scope};
use crate::{
	pattern::{CallPattern, PatternDesc},
	rdf::{EvalDataset, RdfContextMut},
	Pattern, Registry, Value,
};

impl<P: Copy + Ord, R: Clone + Ord> Pattern<P, R> {
	pub fn eval<'s>(
		&'s self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &mut impl RdfContextMut<Resource = R>,
		dataset: &mut impl EvalDataset<R>,
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
		dataset: &mut impl EvalDataset<R>,
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
		dataset: &mut impl EvalDataset<R>,
		scope: &'s Scope<'s, P, R>,
		result: &mut Vec<Value<R>>,
		value: Value<R>,
	) -> Result<(), EvalError<R>> {
		match (self, value) {
			(Self::Var(_), value) => {
				result.push(value);
				Ok(())
			}
			(Self::List(patterns), Value::List(values)) => {
				if patterns.len() != values.len() {
					return Err(EvalError::InvalidType);
				}

				for (p, v) in patterns.iter().zip(values) {
					p.eval(registry, rdf, dataset, scope, v)?;
				}

				Ok(())
			}
			(Self::Map(patterns), Value::Map(entries)) => {
				if patterns.len() != entries.len() {
					return Err(EvalError::InvalidType);
				}

				for (k, v) in entries {
					match patterns.get_untyped(&k) {
						Some(p) => {
							p.eval(registry, rdf, dataset, scope, v)?;
						}
						None => return Err(EvalError::InvalidType),
					}
				}

				Ok(())
			}
			(Self::Call(call), value) => call.eval(registry, rdf, dataset, scope, result, value),
			_ => Err(EvalError::InvalidType),
		}
	}

	pub fn eval_inverse<'s>(
		&'s self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &mut impl RdfContextMut<Resource = R>,
		dataset: &mut impl EvalDataset<R>,
		scope: &'s Scope<'s, P, R>,
		values: &mut Vec<Value<R>>,
	) -> Result<Value<R>, EvalError<R>> {
		match self {
			Self::Var(_) => Ok(values.pop().unwrap()),
			Self::List(patterns) => patterns
				.iter()
				.map(|p| p.eval_inverse(registry, rdf, dataset, scope, values))
				.collect::<Result<_, _>>()
				.map(Value::List),
			Self::Map(patterns) => patterns
				.iter()
				.map(|(k, p)| {
					p.eval_inverse(registry, rdf, dataset, scope, values)
						.map(|v| (k.to_untyped(), v))
				})
				.collect::<Result<_, _>>()
				.map(Value::Map),
			Self::Call(call) => call.eval_inverse(registry, rdf, dataset, scope, values),
		}
	}
}

impl<P: Copy + Ord, R: Clone + Ord> CallPattern<P, R> {
	pub fn eval<'s>(
		&'s self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &mut impl RdfContextMut<Resource = R>,
		dataset: &mut impl EvalDataset<R>,
		scope: &'s Scope<'s, P, R>,
		result: &mut Vec<Value<R>>,
		output: Value<R>,
	) -> Result<(), EvalError<R>> {
		let const_args: Vec<_> = self
			.const_args
			.iter()
			.map(|a| a.eval(registry, rdf, dataset, scope))
			.collect::<Result<_, _>>()?;

		let values = registry
			.get_function(self.function)
			.unwrap()
			.call_inverse_without_unification(registry, rdf, dataset, &const_args, &output)?;

		// Calling the inverse function may cause some resources to be unified
		// together. To ensure that pattern matching remains sound, we must
		// update the RDF dataset accordingly.
		dataset.apply_unification(rdf.interpretation());

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
		dataset: &mut impl EvalDataset<R>,
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

		// Before we can call the function, we must endure that RDF resource
		// unification are reflected in the RDF dataset so that pattern matching
		// remains sound.
		dataset.apply_unification(rdf.interpretation());

		registry
			.get_function(self.function)
			.unwrap()
			.call_without_unification(registry, rdf, dataset, &evaluated_args)
	}
}
