use std::borrow::Cow;

use rdf_types::dataset::{BTreeDataset, PatternMatchingDataset, TraversableDataset};

use crate::{
	expr::Bound,
	pattern::Substitution,
	utils::{QuadsExt, QuadsWithDefaultGraph},
	Matching, TermPattern, Value,
};

use super::{EvalError, RdfContextMut, ReverseScope, Scope, ScopeTypes};

impl<R: Clone + PartialEq> Bound<R> {
	pub fn find_one<'s>(
		&'s self,
		dataset: &impl PatternMatchingDataset<Resource = R>,
		scope: &'s Scope<'s, R>,
	) -> Result<Option<Cow<'s, Scope<'s, R>>>, EvalError<R>> {
		self.find_one_with(dataset, scope, |_| None)
	}

	pub fn find_one_with<'s>(
		&'s self,
		dataset: &impl PatternMatchingDataset<Resource = R>,
		scope: &'s Scope<'s, R>,
		f: impl Fn(u32) -> Option<R>,
	) -> Result<Option<Cow<'s, Scope<'s, R>>>, EvalError<R>> {
		if self.intro.is_empty() {
			Ok(Some(Cow::Borrowed(scope)))
		} else {
			let mut matching = Matching::new(
				dataset,
				scope,
				Substitution::new(self.intro.len() as u32, |i| f(i).map(Value::Resource)),
				self.dataset.quads().with_default_graph(scope.graph()),
			);

			match matching.next() {
				Some(values) => {
					if matching.next().is_some() {
						Err(EvalError::Ambiguity)
					} else {
						Ok(Some(Cow::Owned(Scope::new(
							Some(scope),
							None,
							ScopeTypes::Slice(&self.intro),
							Cow::Owned(values),
						))))
					}
				}
				None => Ok(None),
			}
		}
	}

	pub fn inverse_once<'a, C>(
		&'a self,
		rdf: &mut C,
		scope: &'a mut ReverseScope<R>,
		f: impl FnOnce(&mut C, &mut ReverseScope<'a, R>) -> Result<(), EvalError<R>>,
	) -> Result<Vec<Value<R>>, EvalError<R>>
	where
		C: RdfContextMut<R>,
	{
		scope.begin(rdf, None, &self.intro, f)
	}

	pub fn find_all<'a, D>(
		&'a self,
		dataset: &'a D,
		scope: &'a Scope<'a, R>,
	) -> Matching<R, D, EmbeddedQuads<'_, R>>
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
}

type EmbeddedQuads<'a, R> =
	QuadsWithDefaultGraph<'a, R, <BTreeDataset<TermPattern<R>> as TraversableDataset>::Quads<'a>>;
