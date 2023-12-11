use grdf::Quad;

use crate::{pattern::Substitution, Pattern};

pub enum Error {
	Ambiguity,
	Empty,
}

pub struct Matching<'a, 'p, R, D, Q>
where
	D: grdf::Dataset<Subject = R, Predicate = R, Object = R, GraphLabel = R>,
{
	dataset: &'a D,
	stack: Vec<State<'a, 'p, R, D, Q>>,
}

pub struct State<'a, 'p, R, D, Q>
where
	D: 'a + grdf::Dataset<Subject = R, Predicate = R, Object = R, GraphLabel = R>,
{
	substitution: Substitution<R>,
	quad_state: Option<QuadState<'a, 'p, R, D>>,
	rest: Q,
}

pub struct QuadState<'a, 'p, R, D>
where
	D: 'a + grdf::Dataset<Subject = R, Predicate = R, Object = R, GraphLabel = R>,
{
	pattern: Quad<Pattern<&'p R>, Pattern<&'p R>, Pattern<&'p R>, Pattern<&'p R>>,
	quad_matching: D::PatternMatching<'a, 'p>,
}

impl<'a, 'p, R, D, Q> Matching<'a, 'p, R, D, Q>
where
	D: grdf::Dataset<Subject = R, Predicate = R, Object = R, GraphLabel = R>,
{
	pub fn new(dataset: &'a D, substitution: Substitution<R>, quads: Q) -> Self {
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

impl<'a, 'p, R, D, Q> Matching<'a, 'p, R, D, Q>
where
	R: Clone + PartialEq,
	D: grdf::Dataset<Subject = R, Predicate = R, Object = R, GraphLabel = R>,
	Q: Clone
		+ Iterator<Item = Quad<Pattern<&'p R>, Pattern<&'p R>, Pattern<&'p R>, Pattern<&'p R>>>,
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
	R: Clone + PartialEq,
	D: grdf::Dataset<Subject = R, Predicate = R, Object = R, GraphLabel = R>,
	Q: Clone
		+ Iterator<Item = Quad<Pattern<&'p R>, Pattern<&'p R>, Pattern<&'p R>, Pattern<&'p R>>>,
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
									.pattern_matching(quad_matching_pattern(pattern)),
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

fn quad_matching_pattern<'p, R>(
	pattern: Quad<Pattern<&'p R>, Pattern<&'p R>, Pattern<&'p R>, Pattern<&'p R>>,
) -> Quad<Option<&'p R>, Option<&'p R>, Option<&'p R>, Option<&'p R>> {
	Quad(
		pattern.0.into_resource(),
		pattern.1.into_resource(),
		pattern.2.into_resource(),
		pattern.3.map(Pattern::into_resource),
	)
}
