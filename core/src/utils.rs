pub mod automaton;

pub use automaton::{Automaton, DetAutomaton};

use btree_range_map::RangeSet;
use educe::Educe;
use rdf_types::Quad;

use crate::Pattern;

pub fn charset_intersection(a: &RangeSet<char>, b: &RangeSet<char>) -> RangeSet<char> {
	let mut result = a.clone();

	for r in b.gaps() {
		result.remove(r.cloned());
	}

	result
}

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
		todo!()
	}
}
