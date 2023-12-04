use educe::Educe;
use rdf_types::Quad;

use crate::Pattern;

pub trait QuadsExt<'a, R>: Sized {
	fn with_default_graph(self, graph: Option<&'a R>) -> QuadsWithDefaultGraph<'a, R, Self>;
}

impl<'a, R: 'a, I> QuadsExt<'a, R> for I
where
	I: Iterator<Item = Quad<&'a Pattern<R>, &'a Pattern<R>, &'a Pattern<R>, &'a Pattern<R>>>,
{
	fn with_default_graph(self, graph: Option<&'a R>) -> QuadsWithDefaultGraph<'a, R, Self> {
		QuadsWithDefaultGraph { quads: self, graph }
	}
}

#[derive(Educe)]
#[educe(Clone(bound = "I: Clone"))]
pub struct QuadsWithDefaultGraph<'a, R, I> {
	quads: I,
	graph: Option<&'a R>,
}

impl<'a, R, I> Iterator for QuadsWithDefaultGraph<'a, R, I>
where
	I: Iterator<Item = Quad<&'a Pattern<R>, &'a Pattern<R>, &'a Pattern<R>, &'a Pattern<R>>>,
{
	type Item = Quad<Pattern<&'a R>, Pattern<&'a R>, Pattern<&'a R>, Pattern<&'a R>>;

	fn next(&mut self) -> Option<Self::Item> {
		self.quads.next().map(|quad| {
			Quad(
				quad.0.as_ref(),
				quad.1.as_ref(),
				quad.2.as_ref(),
				quad.3
					.map(Pattern::as_ref)
					.or_else(|| self.graph.map(Pattern::Resource)),
			)
		})
	}
}
