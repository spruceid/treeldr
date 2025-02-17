use std::borrow::Cow;

use rdf_types::{
	dataset::{BTreeDataset, TraversableDataset},
	Quad,
};

use crate::{
	expr::Bindings,
	pattern::Substitution,
	utils::{QuadsExt, QuadsWithDefaultGraph},
	Matching, Registry, TermPattern, Value,
};

use super::{Eval, EvalContext, EvalError, Scope, ScopeTypes, ScopeValues};

impl<P: Copy + Ord, R: Clone + Ord> Bindings<P, R> {
	pub fn eval<'s>(
		&'s self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &mut impl EvalContext<Resource = R>,
		scope: &'s Scope<'s, P, R>,
	) -> Result<Option<Cow<'s, Scope<'s, P, R>>>, EvalError<R>> {
		let mut values = Vec::with_capacity(self.len());

		for b in &self.list {
			match &b.assignment {
				Some(e) => {
					let value = e.eval(registry, rdf, scope)?;
					values.extend(
						b.pattern
							.eval(registry, rdf, scope, value)?
							.into_iter()
							.map(Some),
					);
				}
				None => {
					b.pattern.visit_variables(|_| values.push(None));
				}
			}
		}

		self.find_one_with(rdf, scope, |i| Option::take(&mut values[i as usize]))
	}

	pub fn eval_inverse<C>(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &mut C,
		scope: &mut Scope<P, R>,
		f: impl FnOnce(&mut C, &mut Scope<P, R>) -> Result<(), EvalError<R>>,
	) -> Result<(), EvalError<R>>
	where
		C: EvalContext<Resource = R>,
	{
		let mut values = scope.begin_mut(
			rdf,
			None,
			ScopeTypes::Slice(&self.variables),
			move |rdf, scope| {
				scope.extend_dataset(self.dataset.iter().map(Quad::into_cloned));
				f(rdf, scope)
			},
		)?;

		// Reverse the values, because patterns need to pop the values front to
		// back.
		values.reverse();

		for b in &self.list {
			let value = b.pattern.eval_inverse(registry, rdf, scope, &mut values)?;
			if let Some(e) = &b.assignment {
				e.eval_inverse(registry, rdf, scope, &value)?;
			}
		}

		Ok(())
	}
}

impl<P: Copy, R: Clone + PartialEq> Bindings<P, R> {
	pub fn find_one_with<'s>(
		&'s self,
		rdf: &impl EvalContext<Resource = R>,
		scope: &'s Scope<'s, P, R>,
		f: impl FnMut(u32) -> Option<Value<R>>,
	) -> Result<Option<Cow<'s, Scope<'s, P, R>>>, EvalError<R>> {
		let len = self.len();
		if len == 0 {
			Ok(Some(Cow::Borrowed(scope)))
		} else {
			let mut matching = Matching::new(
				rdf,
				scope,
				Substitution::new(len as u32, f),
				self.dataset
					.quads()
					.map(|quad| quad.map(TermPattern::as_ref))
					.with_default_graph(scope.graph().map(TermPattern::Resource)),
			);

			match matching.next() {
				Some(values) => {
					if matching.next().is_some() {
						Err(EvalError::Ambiguity)
					} else {
						Ok(Some(Cow::Owned(scope.begin(
							None,
							ScopeTypes::Slice(&self.variables),
							ScopeValues::ConstOwned(values),
						))))
					}
				}
				None => Ok(None),
			}
		}
	}

	pub fn find_all<'a, D>(
		&'a self,
		rdf: &'a D,
		scope: &'a Scope<'a, P, R>,
	) -> Matching<'a, P, R, D, EmbeddedQuads<'a, R>>
	where
		D: EvalContext<Resource = R>,
	{
		let quad_pattern_as_ref: QuadPatternAsRef<'a, R> = |quad| quad.map(TermPattern::as_ref);
		Matching::new(
			rdf,
			scope,
			Substitution::new(self.list.len() as u32, |_| None),
			self.dataset
				.quads()
				.map(quad_pattern_as_ref)
				.with_default_graph(scope.graph().map(TermPattern::Resource)),
		)
	}

	pub fn find_one_inverse<C>(
		&self,
		rdf: &mut C,
		scope: &mut Scope<P, R>,
		f: impl FnOnce(&mut C, &mut Scope<P, R>) -> Result<(), EvalError<R>>,
	) -> Result<Vec<Value<R>>, EvalError<R>>
	where
		C: EvalContext<Resource = R>,
	{
		let values = scope.begin_mut(
			rdf,
			None,
			ScopeTypes::Slice(&self.variables),
			move |rdf, scope| {
				let quads = self.dataset.iter().map(Quad::into_cloned);
				scope.extend_dataset(quads);
				f(rdf, scope)
			},
		);

		values
	}
}

// fn binding_quad_to_deduced<'a, R>(quad: Quad<&'a TermPattern<R>>) -> Quad<TermPattern<DeducedResourceRef<'a, R>>> {
// 	quad.map(|pattern| {
// 		pattern.mapped(|r| DeducedResourceRef::Base(r))
// 	})
// }

type QuadPatternAsRef<'a, R> = fn(Quad<&'a TermPattern<R>>) -> Quad<TermPattern<&'a R>>;

type BindingQuads<'a, R> = std::iter::Map<
	<BTreeDataset<TermPattern<R>> as TraversableDataset>::Quads<'a>,
	QuadPatternAsRef<'a, R>,
>;

type EmbeddedQuads<'a, R> = QuadsWithDefaultGraph<TermPattern<&'a R>, BindingQuads<'a, R>>;
