use std::marker::PhantomData;

use rdf_types::{
	dataset::{DatasetMut, PatternMatchingDataset},
	pattern::CanonicalQuadPattern,
	Dataset, Quad,
};

use crate::{expr::Call, Registry, Value};

use super::{Eval, EvalError, RdfContext, RdfContextMut, Scope};

impl<R: Clone + Ord> Eval<R> for Call<R> {
	fn eval(
		&self,
		registry: &Registry<R>,
		rdf: &impl RdfContext<R>,
		dataset: &impl PatternMatchingDataset<Resource = R>,
		scope: &Scope<R>,
	) -> Result<Value<R>, EvalError<R>> {
		let mut evaluated_args = Vec::with_capacity(self.const_args.len() + self.args.len());

		for a in &self.const_args {
			evaluated_args.push(a.eval(registry, rdf, dataset, scope)?);
		}

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
		scope: &mut Scope<R>,
		output: &Value<R>,
	) -> Result<(), EvalError<R>> {
		let const_args: Vec<_> = self
			.const_args
			.iter()
			.map(|a| a.eval(registry, rdf, &InaccessibleDataset::default(), scope))
			.collect::<Result<_, _>>()?;

		let values = registry.get_function(self.function).unwrap().call_inverse(
			registry,
			rdf,
			dataset,
			&const_args,
			output,
		)?;

		for (arg, value) in self.args.iter().zip(values) {
			arg.eval_inverse(registry, rdf, dataset, scope, &value)?;
		}

		Ok(())
	}
}

struct InaccessibleDataset<R>(PhantomData<R>);

impl<R> Default for InaccessibleDataset<R> {
	fn default() -> Self {
		Self(PhantomData)
	}
}

impl<R> Dataset for InaccessibleDataset<R> {
	type Resource = R;
}

impl<R> PatternMatchingDataset for InaccessibleDataset<R> {
	type QuadPatternMatching<'a, 'p> = InaccessibleQuadPatternMatching<'a, R> where
	Self: 'a,
	Self::Resource: 'p;

	fn quad_pattern_matching<'p>(
		&self,
		_pattern: CanonicalQuadPattern<&'p Self::Resource>,
	) -> Self::QuadPatternMatching<'_, 'p> {
		InaccessibleQuadPatternMatching(PhantomData)
	}
}

struct InaccessibleQuadPatternMatching<'a, R>(PhantomData<&'a R>);

impl<'a, R> Iterator for InaccessibleQuadPatternMatching<'a, R> {
	type Item = Quad<&'a R>;

	fn next(&mut self) -> Option<Self::Item> {
		None
	}
}
