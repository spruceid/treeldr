use std::borrow::Cow;

use rdf_types::{
	dataset::{BTreeDataset, DatasetMut, PatternMatchingDataset, TraversableDataset},
	Quad,
};

use crate::{
	expr::Bindings,
	pattern::Substitution,
	rdf::EvalDataset,
	utils::{QuadsExt, QuadsWithDefaultGraph},
	Matching, Registry, TermPattern, Value,
};

use super::{Eval, EvalError, RdfContextMut, Scope, ScopeTypes, ScopeValues};

impl<P: Copy + Ord, R: Clone + Ord> Bindings<P, R> {
	pub fn eval<'s>(
		&'s self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &mut impl RdfContextMut<Resource = R>,
		dataset: &mut impl EvalDataset<R>,
		scope: &'s Scope<'s, P, R>,
	) -> Result<Option<Cow<'s, Scope<'s, P, R>>>, EvalError<R>> {
		let mut values = Vec::with_capacity(self.len());

		for b in &self.list {
			match &b.assignment {
				Some(e) => {
					let value = e.eval(registry, rdf, dataset, scope)?;
					values.extend(
						b.pattern
							.eval(registry, rdf, dataset, scope, value)?
							.into_iter()
							.map(Some),
					);
				}
				None => {
					b.pattern.visit_variables(|_| values.push(None));
				}
			}
		}

		self.find_one_with(dataset, scope, |i| values[i as usize].take())
	}

	pub fn eval_inverse<C, D>(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &mut C,
		dataset: &mut D,
		scope: &mut Scope<P, R>,
		f: impl FnOnce(&mut C, &mut D, &mut Scope<P, R>) -> Result<(), EvalError<R>>,
	) -> Result<(), EvalError<R>>
	where
		C: RdfContextMut<Resource = R>,
		D: EvalDataset<R>,
	{
		let mut values = scope.begin_mut(
			rdf,
			dataset,
			None,
			ScopeTypes::Slice(&self.variables),
			move |rdf, dataset, scope| {
				scope.extend_dataset(self.dataset.iter().map(Quad::into_cloned));
				f(rdf, dataset, scope)
			},
		)?;

		// Reverse the values, because patterns need to pop the values front to
		// back.
		values.reverse();

		for b in &self.list {
			let value = b
				.pattern
				.eval_inverse(registry, rdf, dataset, scope, &mut values)?;
			if let Some(e) = &b.assignment {
				e.eval_inverse(registry, rdf, dataset, scope, &value)?;
			}
		}

		Ok(())
	}
}

impl<P: Copy, R: Clone + PartialEq> Bindings<P, R> {
	pub fn find_one_with<'s>(
		&'s self,
		dataset: &impl PatternMatchingDataset<Resource = R>,
		scope: &'s Scope<'s, P, R>,
		// f: impl Fn(u32) -> Result<Option<Value<R>>, EvalError<R>>,
		f: impl FnMut(u32) -> Option<Value<R>>,
	) -> Result<Option<Cow<'s, Scope<'s, P, R>>>, EvalError<R>> {
		let len = self.len();
		if len == 0 {
			Ok(Some(Cow::Borrowed(scope)))
		} else {
			let mut matching = Matching::new(
				dataset,
				scope,
				Substitution::new(len as u32, f),
				self.dataset.quads().with_default_graph(scope.graph()),
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
		dataset: &'a D,
		scope: &'a Scope<'a, P, R>,
	) -> Matching<'a, 'a, P, R, D, EmbeddedQuads<'a, R>>
	where
		D: PatternMatchingDataset<Resource = R>,
	{
		Matching::new(
			dataset,
			scope,
			Substitution::new(self.list.len() as u32, |_| None),
			self.dataset.quads().with_default_graph(scope.graph()),
		)
	}

	pub fn find_one_inverse<C, D>(
		&self,
		rdf: &mut C,
		dataset: &mut D,
		scope: &mut Scope<P, R>,
		f: impl FnOnce(&mut C, &mut D, &mut Scope<P, R>) -> Result<(), EvalError<R>>,
	) -> Result<Vec<Value<R>>, EvalError<R>>
	where
		C: RdfContextMut<Resource = R>,
		D: DatasetMut<Resource = R>,
	{
		let values = scope.begin_mut(
			rdf,
			dataset,
			None,
			ScopeTypes::Slice(&self.variables),
			move |rdf, dataset, scope| {
				scope.extend_dataset(self.dataset.iter().map(Quad::into_cloned));
				f(rdf, dataset, scope)
			},
		);

		values
	}
}

type EmbeddedQuads<'a, R> =
	QuadsWithDefaultGraph<'a, R, <BTreeDataset<TermPattern<R>> as TraversableDataset>::Quads<'a>>;
