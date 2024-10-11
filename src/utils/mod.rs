
use educe::Educe;
use iref::IriBuf;
use locspan::{Meta, Span};
use rdf_types::{GraphLabel, Id, Object, Quad, Term};

use crate::TermPattern;

mod ignored;
pub use ignored::*;

pub trait QuadsExt<'a, R>: Sized {
	fn with_default_graph(self, graph: Option<&'a R>) -> QuadsWithDefaultGraph<'a, R, Self>;
}

impl<'a, R: 'a, I> QuadsExt<'a, R> for I
where
	I: Iterator<
		Item = Quad<&'a TermPattern<R>, &'a TermPattern<R>, &'a TermPattern<R>, &'a TermPattern<R>>,
	>,
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
	I: Iterator<
		Item = Quad<&'a TermPattern<R>, &'a TermPattern<R>, &'a TermPattern<R>, &'a TermPattern<R>>,
	>,
{
	type Item =
		Quad<TermPattern<&'a R>, TermPattern<&'a R>, TermPattern<&'a R>, TermPattern<&'a R>>;

	fn next(&mut self) -> Option<Self::Item> {
		self.quads.next().map(|quad| {
			Quad(
				quad.0.as_ref(),
				quad.1.as_ref(),
				quad.2.as_ref(),
				quad.3
					.map(TermPattern::as_ref)
					.or_else(|| self.graph.map(TermPattern::Resource)),
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

pub fn try_map_list<A, B>(
	l: Vec<A>,
	revert: impl Fn(B) -> A,
	f: impl Fn(usize, A) -> Result<B, A>,
) -> Result<Vec<B>, Vec<A>> {
	let len = l.len();
	let mut result = Vec::with_capacity(len);

	let mut items = l.into_iter();

	let mut i = 0;
	while let Some(item) = items.next() {
		match f(i, item) {
			Ok(b) => {
				result.push(b);
				i += 1
			}
			Err(a) => {
				let mut reverted = Vec::with_capacity(len);
				reverted.extend(result.into_iter().map(&revert));
				reverted.push(a);
				reverted.extend(items);
				return Err(reverted);
			}
		}
	}

	Ok(result)
}

pub fn take_first<T>(list: &mut Vec<T>, mut predicate: impl FnMut(&T) -> bool) -> Option<T> {
	for i in 0..list.len() {
		if predicate(&list[i]) {
			return Some(list.swap_remove(i));
		}
	}

	None
}
