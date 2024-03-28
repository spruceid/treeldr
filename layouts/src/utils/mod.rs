pub mod automaton;

pub use automaton::{Automaton, DetAutomaton};

use btree_range_map::RangeSet;
use educe::Educe;
use iref::IriBuf;
use locspan::{Meta, Span};
use rdf_types::{GraphLabel, Id, Object, Quad, Term};

use crate::Pattern;

/// Computes the intersection of two character sets.
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

pub type MetaQuad = Meta<
	rdf_types::Quad<Meta<Id, Span>, Meta<IriBuf, Span>, Meta<Object, Span>, Meta<GraphLabel, Span>>,
	Span,
>;

/// Strips the input RDF `quad` of its metadata information and returns it as a
/// gRDF quad (a quad where all components are [`Term`](rdf_types::Term)s).
pub fn strip_rdf_quad(
	locspan::Meta(Quad(
		locspan::Meta(s, _),
		locspan::Meta(p, _),
		locspan::Meta(o, _),
		g
	), _): MetaQuad,
) -> Quad {
	Quad(
		Term::Id(s),
		Term::iri(p),
		o,
		g.map(|locspan::Meta(g, _)| Term::Id(g)),
	)
}
