use rdf_types::{Triple, Object, Generator, Namespace};

use crate::RdfIterator;

mod literal;

pub use literal::*;

use super::iter;

/// Triple or value sum type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TripleOrValue<I, L> {
	Triple(Triple<I, I, Object<I, L>>),
	Value(Object<I, L>)
}

pub trait TriplesAndValues<N: Namespace, L> {
	type TriplesAndValues<'a>: RdfIterator<N, Item = TripleOrValue<N::Id, L>> where Self: 'a;

	fn unbound_rdf_triples_and_values<G: Generator<N>>(
		&self,
		namespace: &mut N,
		generator: &mut G
	) -> Self::TriplesAndValues<'_>;

	fn rdf_triples_and_values<'n, 'g, G: Generator<N>>(
		&self,
		namespace: &'n mut N,
		generator: &'g mut G
	) -> iter::Bound<'n, 'g, Self::TriplesAndValues<'_>, N, G> {
		let inner = self.unbound_rdf_triples_and_values(namespace, generator);
		iter::Bound::new(inner, namespace, generator)
	}
}

impl<T: TriplesAndValues<N, L>, N: Namespace, L> TriplesAndValues<N, L> for Option<T> {
	type TriplesAndValues<'a> = super::iter::Optional<T::TriplesAndValues<'a>> where Self: 'a;

	fn unbound_rdf_triples_and_values<G: Generator<N>>(
		&self,
		namespace: &mut N,
		generator: &mut G
	) -> Self::TriplesAndValues<'_> {
		super::iter::Optional::new(self.as_ref().map(|t| t.unbound_rdf_triples_and_values(namespace, generator)))
	}
}

pub trait Triples<N: Namespace, L> {
	type Triples<'a>: 'a + RdfIterator<N, Item = Triple<N::Id, N::Id, Object<N::Id, L>>> where Self: 'a;

	fn unbound_rdf_triples<'a, G: Generator<N>>(
		&self,
		namespace: &mut N,
		generator: &mut G
	) -> Self::Triples<'_>;

	fn rdf_triples<'n, 'g, G: Generator<N>>(
		&self,
		namespace: &'n mut N,
		generator: &'g mut G
	) -> iter::Bound<'n, 'g, Self::Triples<'_>, N, G> {
		let inner = self.unbound_rdf_triples(namespace, generator);
		iter::Bound::new(inner, namespace, generator)
	}
}

/// Wrapper that changes a `TripleOrValue<I, L>` iterator into a
/// `Triple<I, I, Object<I, L>>` iterator.
pub struct FilterTriples<T>(pub T);

impl<N: Namespace, L, T: RdfIterator<N, Item = TripleOrValue<N::Id, L>>> RdfIterator<N> for FilterTriples<T> {
	type Item = Triple<N::Id, N::Id, Object<N::Id, L>>;

	fn next_with<'n, 'g, G: Generator<N>>(
		&mut self,
		namespace: &'n mut N,
		generator: &'g mut G
	) -> Option<Self::Item> {
		loop {
			match self.0.next_with(namespace, generator) {
				Some(TripleOrValue::Triple(triple)) => break Some(triple),
				Some(TripleOrValue::Value(_)) => (),
				None => break None
			}
		}
	}
}