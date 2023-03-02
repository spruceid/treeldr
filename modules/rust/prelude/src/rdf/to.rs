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

/// RDF traversal.
pub trait TriplesAndValues<N: Namespace, L> {
	type TriplesAndValues<'a>: 'a + RdfIterator<N, Item = TripleOrValue<N::Id, L>> where Self: 'a, N::Id: 'a, L: 'a;

	fn unbound_rdf_triples_and_values<'a, G: Generator<N>>(
		&'a self,
		namespace: &mut N,
		generator: &mut G
	) -> Self::TriplesAndValues<'a> where N::Id: 'a, L: 'a;

	fn rdf_triples_and_values<'a, 'n, 'g, G: Generator<N>>(
		&'a self,
		namespace: &'n mut N,
		generator: &'g mut G
	) -> iter::Bound<'n, 'g, Self::TriplesAndValues<'a>, N, G> where N::Id: 'a, L: 'a {
		let inner = self.unbound_rdf_triples_and_values(namespace, generator);
		iter::Bound::new(inner, namespace, generator)
	}
}

impl<T: TriplesAndValues<N, L>, N: Namespace, L> TriplesAndValues<N, L> for Option<T> {
	type TriplesAndValues<'a> = super::iter::Optional<T::TriplesAndValues<'a>> where Self: 'a, N::Id: 'a, L: 'a;

	fn unbound_rdf_triples_and_values<'a, G: Generator<N>>(
		&'a self,
		namespace: &mut N,
		generator: &mut G
	) -> Self::TriplesAndValues<'a> where N::Id: 'a, L: 'a {
		super::iter::Optional::new(self.as_ref().map(|t| t.unbound_rdf_triples_and_values(namespace, generator)))
	}
}

/// RDF triples iterator provider.
/// 
/// The namespace `N` defines the node identifier type.
/// The type parameter `L` is the type of literal values.
pub trait Triples<N: Namespace, L> {
	/// Triples iterator.
	type Triples<'a>: 'a + RdfIterator<N, Item = Triple<N::Id, N::Id, Object<N::Id, L>>> where Self: 'a, N::Id: 'a, L: 'a;

	fn unbound_rdf_triples<'a, G: Generator<N>>(
		&'a self,
		namespace: &mut N,
		generator: &mut G
	) -> Self::Triples<'_> where N::Id: 'a, L: 'a;

	fn rdf_triples<'a, 'n, 'g, G: Generator<N>>(
		&'a self,
		namespace: &'n mut N,
		generator: &'g mut G
	) -> iter::Bound<'n, 'g, Self::Triples<'_>, N, G> where N::Id: 'a, L: 'a {
		let inner = self.unbound_rdf_triples(namespace, generator);
		iter::Bound::new(inner, namespace, generator)
	}
}

impl<T: TriplesAndValues<N, L>, N: Namespace, L> Triples<N, L> for T {
	type Triples<'a> = FilterTriples<T::TriplesAndValues<'a>> where Self: 'a, N::Id: 'a, L: 'a;

	fn unbound_rdf_triples<'a, G: Generator<N>>(
		&'a self,
		namespace: &mut N,
		generator: &mut G
	) -> Self::Triples<'a> where N::Id: 'a, L: 'a {
		FilterTriples(self.unbound_rdf_triples_and_values(namespace, generator))
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