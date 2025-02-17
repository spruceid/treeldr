use rdf_types::Quad;

mod ignored;
pub use ignored::*;

pub trait PairExt<A, B> {
	fn map_pair<T>(self, f: impl Fn(B) -> T + Fn(A) -> T) -> (T, T);
}

impl<A, B> PairExt<A, B> for (A, B) {
	fn map_pair<T>(self, f: impl Fn(B) -> T + Fn(A) -> T) -> (T, T) {
		(f(self.0), f(self.1))
	}
}

pub trait QuadsExt<R>: Sized {
	fn with_default_graph(self, graph: Option<R>) -> QuadsWithDefaultGraph<R, Self>;
}

impl<R, I> QuadsExt<R> for I
where
	I: Iterator<Item = Quad<R>>,
{
	fn with_default_graph(self, graph: Option<R>) -> QuadsWithDefaultGraph<R, Self> {
		QuadsWithDefaultGraph { quads: self, graph }
	}
}

#[derive(Debug, Clone)]
pub struct QuadsWithDefaultGraph<R, I> {
	quads: I,
	graph: Option<R>,
}

impl<R, I> Iterator for QuadsWithDefaultGraph<R, I>
where
	R: Clone,
	I: Iterator<Item = Quad<R>>,
{
	type Item = Quad<R>;

	fn next(&mut self) -> Option<Self::Item> {
		self.quads
			.next()
			.map(|quad| quad.map_graph(|g| g.or_else(|| self.graph.clone())))
	}
}

// pub type MetaQuad = Meta<
// 	rdf_types::Quad<Meta<Id, Span>, Meta<IriBuf, Span>, Meta<Object, Span>, Meta<GraphLabel, Span>>,
// 	Span,
// >;

// /// Strips the input RDF `quad` of its metadata information and returns it as a
// /// gRDF quad (a quad where all components are [`Term`](rdf_types::Term)s).
// pub fn strip_rdf_quad(
// 	locspan::Meta(Quad(
// 		locspan::Meta(s, _),
// 		locspan::Meta(p, _),
// 		locspan::Meta(o, _),
// 		g
// 	), _): MetaQuad,
// ) -> Quad {
// 	Quad(
// 		Term::Id(s),
// 		Term::iri(p),
// 		o,
// 		g.map(|locspan::Meta(g, _)| Term::Id(g)),
// 	)
// }

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
