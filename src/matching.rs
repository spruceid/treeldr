use rdf_types::{dataset::PatternMatchingDataset, pattern::CanonicalQuadPattern, Quad};

use crate::{
	eval::Scope,
	pattern::{PatternRefQuad, Substitution},
	TermPattern, Value,
};

/// Pattern matching error.
#[derive(Debug, thiserror::Error)]
pub enum Error {
	/// The input pattern matches more than one fragment of the dataset, where
	/// the caller requires at most one match.
	#[error("ambiguous selection")]
	Ambiguity,

	/// The input pattern does not match any fragment of the dataset, where the
	/// caller requires at least one match.
	#[error("empty selection")]
	Empty,
}

/// Pattern matching.
///
/// Iterates over all the substitutions mapping the input pattern to a fragment
/// of the given RDF dataset.
pub struct Matching<'a, 'p, R: Clone, D, Q>
where
	D: PatternMatchingDataset<Resource = R>,
{
	/// Dataset on which the pattern matching is performed.
	dataset: &'a D,

	/// Working stack.
	stack: Vec<State<'a, 'p, R, D, Q>>,
}

pub struct State<'a, 'p, R: Clone, D, Q>
where
	D: 'a + PatternMatchingDataset<Resource = R>,
{
	scope: MatchingScope<'a, R>,
	quad_state: Option<QuadState<'a, 'p, R, D>>,
	rest: Q,
}

pub struct QuadState<'a, 'p, R, D>
where
	D: 'a + PatternMatchingDataset<Resource = R>,
{
	pattern: PatternRefQuad<'p, R>,
	quad_matching: D::QuadPatternMatching<'a, 'p>,
}

impl<'a, 'p, R: Clone, D, Q> Matching<'a, 'p, R, D, Q>
where
	D: PatternMatchingDataset<Resource = R>,
{
	/// Starts a pattern matching on the given `dataset` using the input partial
	/// `substitution` and `patterns` (an iterator of [`PatternRefQuad`]).
	///
	/// This will return an iterator over all the complete substitutions such
	/// that the substituted patterns form an existing fragment of the dataset.
	///
	/// All the variables appearing in `patterns` *must* be declared in the
	/// initial partial `substitution`. If a variable is already bound in the
	/// initial substitution, only the substitutions preserving the same bound
	/// will be iterated over.
	pub fn new(
		dataset: &'a D,
		scope: &'a Scope<'a, R>,
		substitution: Substitution<R>,
		patterns: Q,
	) -> Self {
		Self {
			dataset,
			stack: vec![State {
				scope: MatchingScope {
					scope,
					substitution,
				},
				quad_state: None,
				rest: patterns,
			}],
		}
	}
}

#[derive(Clone)]
struct MatchingScope<'a, R: Clone> {
	scope: &'a Scope<'a, R>,
	substitution: Substitution<R>,
}

impl<'a, R: Clone + PartialEq> MatchingScope<'a, R> {
	// pub fn assign(&mut self, i: u32, value: TypedValue<R>) -> Result<(), TypedValue<R>> {
	// 	let scope_len = self.scope.len() as u32;
	// 	if i < scope_len {
	// 		if self.scope.get(i).is_some_and(|r| *r == value) {
	// 			Ok(())
	// 		} else {
	// 			Err(value)
	// 		}
	// 	} else {
	// 		let j = i - scope_len;
	// 		if self.substitution.get(j).is_some_and(|x| x == &value) {
	// 			return Err(value)
	// 		}

	// 		self.substitution.set(j, value);
	// 		Ok(())
	// 	}
	// }

	pub fn assign_with(&mut self, i: u32, f: impl FnOnce() -> Value<R>) -> Result<(), Value<R>> {
		let value = f();
		let scope_len = self.scope.len() as u32;
		if i < scope_len {
			if self.scope.get(i).is_some_and(|r| *r == value) {
				Ok(())
			} else {
				Err(value)
			}
		} else {
			let j = i - scope_len;
			self.substitution.set(j, value)
		}
	}

	/// Copies and sets the variables of the substitution by matching the
	/// `value` quad against the given `pattern` quad.
	///
	/// For each `pattern` quad component:
	///   - if the pattern is a variable, sets the variable to the equivalent
	///     resource in `value` using [`Self::set`]. If the variable was already
	///     bound to another resource, a mismatch is found.
	///   - if the pattern is a resource, checks that it is equal to the
	///     corresponding resource in `value`, otherwise a mismatch is found.
	///
	/// Return the updated substitution if no mismatch is found, or `None`
	/// otherwise.
	pub fn with_quad(
		&self,
		pattern: Quad<TermPattern<&R>, TermPattern<&R>, TermPattern<&R>, TermPattern<&R>>,
		value: Quad<&R, &R, &R, &R>,
	) -> Option<Self>
	where
		R: Clone + PartialEq,
	{
		let mut result = self.clone();

		if let TermPattern::Var(x) = pattern.0 {
			if result
				.assign_with(x, || Value::Resource(value.0.clone()))
				.is_err()
			{
				return None;
			}
		}

		if let TermPattern::Var(x) = pattern.1 {
			if result
				.assign_with(x, || Value::Resource(value.1.clone()))
				.is_err()
			{
				return None;
			}
		}

		if let TermPattern::Var(x) = pattern.2 {
			if result
				.assign_with(x, || Value::Resource(value.2.clone()))
				.is_err()
			{
				return None;
			}
		}

		if let Some(TermPattern::Var(x)) = pattern.3 {
			let g = value.3.unwrap();
			if result
				.assign_with(x, || Value::Resource(g.clone()))
				.is_err()
			{
				return None;
			}
		}

		Some(result)
	}
}

impl<'a, 'p, R, D, Q> Matching<'a, 'p, R, D, Q>
where
	R: Clone + PartialEq + 'a,
	D: PatternMatchingDataset<Resource = R>,
	Q: Clone + Iterator<Item = PatternRefQuad<'p, R>>,
{
	pub fn into_optional(mut self) -> Result<Option<Vec<Value<R>>>, Error> {
		match self.next() {
			Some(substitution) => {
				if self.next().is_some() {
					Err(Error::Ambiguity)
				} else {
					Ok(Some(substitution))
				}
			}
			None => Ok(None),
		}
	}

	pub fn into_required(self) -> Result<Vec<Value<R>>, Error> {
		self.into_optional()?.ok_or(Error::Empty)
	}
}

impl<'a, 'p, R, D, Q> Iterator for Matching<'a, 'p, R, D, Q>
where
	R: Clone + PartialEq + 'a,
	D: PatternMatchingDataset<Resource = R>,
	Q: Clone + Iterator<Item = PatternRefQuad<'p, R>>,
{
	type Item = Vec<Value<R>>;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			match self.stack.last_mut() {
				Some(state) => match &mut state.quad_state {
					Some(quad_state) => match quad_state.quad_matching.next() {
						Some(m) => {
							if let Some(scope) = state.scope.with_quad(quad_state.pattern, m) {
								let rest = state.rest.clone();

								self.stack.push(State {
									scope,
									quad_state: None,
									rest,
								})
							}
						}
						None => {
							self.stack.pop();
						}
					},
					None => match state.rest.next() {
						Some(pattern) => {
							state.quad_state = Some(QuadState {
								pattern,
								quad_matching: self
									.dataset
									.quad_pattern_matching(quad_matching_pattern(pattern)),
							})
						}
						None => {
							let state = self.stack.pop().unwrap();
							break Some(state.scope.substitution.into_total().unwrap()); // TODO check that it's always total?
						}
					},
				},
				None => break None,
			}
		}
	}
}

fn quad_matching_pattern<R>(pattern: PatternRefQuad<R>) -> CanonicalQuadPattern<&R> {
	CanonicalQuadPattern::from_pattern(Quad(
		pattern.0.into(),
		pattern.1.into(),
		pattern.2.into(),
		pattern.3.map(Into::into),
	))
}
