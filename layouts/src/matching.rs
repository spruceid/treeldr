use rdf_types::{dataset::PatternMatchingDataset, pattern::CanonicalQuadPattern, Quad};

use crate::pattern::{PatternRefQuad, Substitution};

/// Pattern matching error.
pub enum Error {
	/// The input pattern matches more than one fragment of the dataset, where
	/// the caller requires at most one match.
	Ambiguity,

	/// The input pattern does not match any fragment of the dataset, where the
	/// caller requires at least one match.
	Empty,
}

/// Pattern matching.
///
/// Iterates over all the substitutions mapping the input pattern to a fragment
/// of the given RDF dataset.
pub struct Matching<'a, 'p, R, D, Q>
where
	D: PatternMatchingDataset<Resource = R>,
{
	/// Dataset on which the pattern matching is performed.
	dataset: &'a D,

	/// Working stack.
	stack: Vec<State<'a, 'p, R, D, Q>>,
}

pub struct State<'a, 'p, R, D, Q>
where
	D: 'a + PatternMatchingDataset<Resource = R>,
{
	substitution: Substitution<R>,
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

impl<'a, R, D, Q> Matching<'a, '_, R, D, Q>
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
	pub fn new(dataset: &'a D, substitution: Substitution<R>, patterns: Q) -> Self {
		Self {
			dataset,
			stack: vec![State {
				substitution,
				quad_state: None,
				rest: patterns,
			}],
		}
	}
}

impl<'a, 'p, R, D, Q> Matching<'a, 'p, R, D, Q>
where
	R: Clone + PartialEq + 'a,
	D: PatternMatchingDataset<Resource = R>,
	Q: Clone + Iterator<Item = PatternRefQuad<'p, R>>,
{
	pub fn into_unique(mut self) -> Result<Option<Substitution<R>>, Error> {
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

	pub fn into_required_unique(self) -> Result<Substitution<R>, Error> {
		self.into_unique()?.ok_or(Error::Empty)
	}
}

impl<'a, 'p, R, D, Q> Iterator for Matching<'a, 'p, R, D, Q>
where
	R: Clone + PartialEq + 'a,
	D: PatternMatchingDataset<Resource = R>,
	Q: Clone + Iterator<Item = PatternRefQuad<'p, R>>,
{
	type Item = Substitution<R>;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			match self.stack.last_mut() {
				Some(state) => match &mut state.quad_state {
					Some(quad_state) => match quad_state.quad_matching.next() {
						Some(m) => {
							if let Some(substitution) =
								state.substitution.with_quad(quad_state.pattern, m)
							{
								let rest = state.rest.clone();

								self.stack.push(State {
									substitution,
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
							break Some(state.substitution);
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
