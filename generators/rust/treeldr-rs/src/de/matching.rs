use rdf_types::{dataset::PatternMatchingDataset, pattern::CanonicalQuadPattern, Quad};

use crate::{pattern::Substitution, Pattern};

pub enum Error {
	Ambiguity,
	Empty,
}

pub struct Matching<'a, 'p, D, Q>
where
	D: PatternMatchingDataset,
{
	dataset: &'a D,
	stack: Vec<State<'a, 'p, D, Q>>,
}

pub struct State<'a, 'p, D, Q>
where
	D: 'a + PatternMatchingDataset,
{
	substitution: Substitution<D::Resource>,
	quad_state: Option<QuadState<'a, 'p, D>>,
	rest: Q,
}

pub struct QuadState<'a, 'p, D>
where
	D: 'a + PatternMatchingDataset,
{
	pattern: Quad<Pattern<&'p D::Resource>>,
	quad_matching: D::QuadPatternMatching<'a, 'p>,
}

impl<'a, 'p, D, Q> Matching<'a, 'p, D, Q>
where
	D: PatternMatchingDataset,
{
	pub fn new(dataset: &'a D, substitution: Substitution<D::Resource>, quads: Q) -> Self {
		Self {
			dataset,
			stack: vec![State {
				substitution,
				quad_state: None,
				rest: quads,
			}],
		}
	}
}

impl<'a, 'p, D, Q> Matching<'a, 'p, D, Q>
where
	D: PatternMatchingDataset,
	D::Resource: Clone + PartialEq,
	Q: Clone + Iterator<Item = Quad<Pattern<&'p D::Resource>>>,
{
	pub fn into_unique(mut self) -> Result<Option<Substitution<D::Resource>>, Error> {
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

	pub fn into_required_unique(self) -> Result<Substitution<D::Resource>, Error> {
		self.into_unique()?.ok_or(Error::Empty)
	}
}

impl<'a, 'p, D, Q> Iterator for Matching<'a, 'p, D, Q>
where
	D: PatternMatchingDataset,
	D::Resource: Clone + PartialEq,
	Q: Clone + Iterator<Item = Quad<Pattern<&'p D::Resource>>>,
{
	type Item = Substitution<D::Resource>;

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

fn quad_matching_pattern<R>(pattern: Quad<Pattern<&R>>) -> CanonicalQuadPattern<&R> {
	CanonicalQuadPattern::from_pattern(Quad(
		pattern.0.into(),
		pattern.1.into(),
		pattern.2.into(),
		pattern.3.map(Into::into),
	))
}
