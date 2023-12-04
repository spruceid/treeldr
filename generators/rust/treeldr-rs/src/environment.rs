use crate::{Pattern, RdfContextMut};
use rdf_types::{InterpretationMut, Quad};

pub enum Environment<'a, R> {
	Root(&'a [R]),
	Child(&'a Environment<'a, R>, Vec<R>),
}

impl<'a, R> Environment<'a, R> {
	pub fn get(&self, i: u32) -> Result<&R, u32> {
		match self {
			Self::Root(inputs) => match inputs.get(i as usize) {
				Some(r) => Ok(r),
				None => Err(i - inputs.len() as u32),
			},
			Self::Child(parent, intros) => match parent.get(i) {
				Ok(r) => Ok(r),
				Err(j) => match intros.get(j as usize) {
					Some(r) => Ok(r),
					None => Err(j - intros.len() as u32),
				},
			},
		}
	}

	#[must_use]
	pub fn bind<const N: usize>(&self, resources: [R; N]) -> Environment<R> {
		Environment::Child(self, resources.into_iter().collect())
	}

	#[must_use]
	pub fn intro<V, I>(&self, rdf: &mut RdfContextMut<V, I>, count: u32) -> Environment<R>
	where
		I: InterpretationMut<V, Resource = R>,
	{
		let mut intros = Vec::with_capacity(count as usize);
		for _ in 0..count {
			intros.push(rdf.interpretation.new_resource(rdf.vocabulary))
		}

		Environment::Child(self, intros)
	}
}

impl<'a, R: Clone> Environment<'a, R> {
	pub fn instantiate_pattern<Q>(&self, pattern: &Pattern<Q>) -> R
	where
		Q: Clone + Into<R>,
	{
		match pattern {
			Pattern::Var(x) => self.get(*x).cloned().unwrap(),
			Pattern::Resource(r) => r.clone().into(),
		}
	}

	pub fn instantiate_patterns<const N: usize>(&self, patterns: &[Pattern<R>; N]) -> [R; N]
	where
		// Q: Clone + Into<R>,
		R: Clone,
	{
		let mut result = Vec::with_capacity(patterns.len());

		for p in patterns {
			result.push(self.instantiate_pattern(p))
		}

		result.try_into().ok().unwrap()
	}

	pub fn instantiate_quad(
		&self,
		quad: Quad<&Pattern<R>, &Pattern<R>, &Pattern<R>, &Pattern<R>>,
	) -> Quad<R, R, R, R>
	where
		// Q: Clone + Into<R>,
		R: Clone,
	{
		Quad(
			self.instantiate_pattern(quad.0),
			self.instantiate_pattern(quad.1),
			self.instantiate_pattern(quad.2),
			quad.3.map(|g| self.instantiate_pattern(g)),
		)
	}

	pub fn instantiate_dataset<D>(
		&self,
		input: &[Quad<Pattern<R>, Pattern<R>, Pattern<R>, Pattern<R>>],
		output: &mut D,
	) where
		// Q: Clone + Into<R>,
		R: Clone,
		D: grdf::MutableDataset<Subject = R, Predicate = R, Object = R, GraphLabel = R>,
	{
		for quad in input {
			output.insert(self.instantiate_quad(quad.borrow_components()));
		}
	}
}
