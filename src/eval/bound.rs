use std::borrow::Cow;

use rdf_types::{
	dataset::{BTreeDataset, DatasetMut, PatternMatchingDataset, TraversableDataset},
	Quad,
};

use crate::{
	expr::Bindings,
	pattern::Substitution,
	rdf::RdfContext,
	utils::{QuadsExt, QuadsWithDefaultGraph},
	Matching, Registry, TermPattern, Value,
};

use super::{Eval, EvalError, RdfContextMut, Scope, ScopeTypes, ScopeValues};

impl<P: Copy + Ord, R: Clone + Ord> Bindings<P, R> {
	pub fn eval<'s>(
		&'s self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &impl RdfContext<Resource = R>,
		dataset: &impl PatternMatchingDataset<Resource = R>,
		scope: &'s Scope<'s, P, R>,
	) -> Result<Option<Cow<'s, Scope<'s, P, R>>>, EvalError<R>> {
		self.find_one_with(dataset, scope, |i| {
			self.intro[i as usize]
				.assignment
				.as_ref()
				.map(|e| e.eval(registry, rdf, dataset, scope))
				.transpose()
		})
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
		D: DatasetMut<Resource = R>,
	{
		let values = scope.begin_mut(
			rdf,
			dataset,
			None,
			ScopeTypes::Bindings(&self.intro),
			move |rdf, dataset, scope| {
				scope.extend_dataset(self.dataset.iter().map(Quad::into_cloned));
				f(rdf, dataset, scope)
			},
		)?;

		for (i, value) in values.iter().enumerate() {
			if let Some(e) = &self.intro[i].assignment {
				e.eval_inverse(registry, rdf, dataset, scope, value)?;
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
		f: impl Fn(u32) -> Result<Option<Value<R>>, EvalError<R>>,
	) -> Result<Option<Cow<'s, Scope<'s, P, R>>>, EvalError<R>> {
		if self.intro.is_empty() {
			Ok(Some(Cow::Borrowed(scope)))
		} else {
			let mut matching = Matching::new(
				dataset,
				scope,
				Substitution::try_new(self.intro.len() as u32, f)?,
				self.dataset.quads().with_default_graph(scope.graph()),
			);

			match matching.next() {
				Some(values) => {
					if matching.next().is_some() {
						Err(EvalError::Ambiguity)
					} else {
						Ok(Some(Cow::Owned(scope.begin(
							None,
							ScopeTypes::Bindings(&self.intro),
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
	) -> Matching<P, R, D, EmbeddedQuads<'_, R>>
	where
		D: PatternMatchingDataset<Resource = R>,
	{
		Matching::new(
			dataset,
			scope,
			Substitution::new(self.intro.len() as u32, |_| None),
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
			ScopeTypes::Bindings(&self.intro),
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
