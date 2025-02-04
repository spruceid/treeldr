use std::marker::PhantomData;

use rdf_types::{dataset::PatternMatchingDataset, pattern::CanonicalQuadPattern, Dataset, Quad};

use crate::{expr::Call, rdf::EvalDataset, Registry, Value};

use super::{Eval, EvalError, RdfContextMut, Scope};

impl<P: Copy + Ord, R: Clone + Ord> Eval<P, R> for Call<P, R> {
	fn eval(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &mut impl RdfContextMut<Resource = R>,
		dataset: &mut impl EvalDataset<R>,
		scope: &Scope<P, R>,
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
			.call_without_unification(registry, rdf, dataset, &evaluated_args)
	}

	fn eval_inverse(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &mut impl RdfContextMut<Resource = R>,
		dataset: &mut impl EvalDataset<R>,
		scope: &mut Scope<P, R>,
		output: &Value<R>,
	) -> Result<(), EvalError<R>> {
		let const_args: Vec<_> = self
			.const_args
			.iter()
			.map(|a| a.eval(registry, rdf, dataset, scope))
			.collect::<Result<_, _>>()?;

		let values = registry
			.get_function(self.function)
			.unwrap()
			.call_inverse_without_unification(registry, rdf, dataset, &const_args, output)?;

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
	type QuadPatternMatching<'a, 'p>
		= InaccessibleQuadPatternMatching<'a, R>
	where
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
