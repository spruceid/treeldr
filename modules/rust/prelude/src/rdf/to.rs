use std::{collections::BTreeSet, marker::PhantomData};

use rdf_types::{Generator, Namespace, Object, Triple};

use crate::{Id, RdfIterator};

mod literal;

pub use literal::*;

use super::iter;

/// Triple or value sum type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TripleOrValue<I, L> {
	Triple(Triple<I, I, Object<I, L>>),
	Value(Object<I, L>),
}

/// RDF traversal.
pub trait TriplesAndValues<N: Namespace, L> {
	type TriplesAndValues<'a>: 'a + RdfIterator<N, Item = TripleOrValue<N::Id, L>>
	where
		Self: 'a,
		N::Id: 'a,
		L: 'a;

	fn unbound_rdf_triples_and_values<'a, G: Generator<N>>(
		&'a self,
		namespace: &mut N,
		generator: &mut G,
	) -> Self::TriplesAndValues<'a>
	where
		N::Id: 'a,
		L: 'a;

	fn rdf_triples_and_values<'a, 'n, 'g, G: Generator<N>>(
		&'a self,
		namespace: &'n mut N,
		generator: &'g mut G,
	) -> iter::Bound<'n, 'g, Self::TriplesAndValues<'a>, N, G>
	where
		N::Id: 'a,
		L: 'a,
	{
		let inner = self.unbound_rdf_triples_and_values(namespace, generator);
		iter::Bound::new(inner, namespace, generator)
	}
}

impl<N: Namespace, L> TriplesAndValues<N, L> for Id<N::Id>
where
	N::Id: Clone,
{
	type TriplesAndValues<'a> = ValuesOnly<IdValue<'a, N::Id, L>> where Self: 'a, N::Id: 'a, L: 'a;

	fn unbound_rdf_triples_and_values<'a, G: Generator<N>>(
		&'a self,
		_namespace: &mut N,
		_generator: &mut G,
	) -> Self::TriplesAndValues<'a>
	where
		N::Id: 'a,
		L: 'a,
	{
		ValuesOnly::new(IdValue::new(&self.0))
	}
}

impl<T: TriplesAndValues<N, L>, N: Namespace, L> TriplesAndValues<N, L> for Option<T> {
	type TriplesAndValues<'a> = super::iter::Optional<T::TriplesAndValues<'a>> where Self: 'a, N::Id: 'a, L: 'a;

	fn unbound_rdf_triples_and_values<'a, G: Generator<N>>(
		&'a self,
		namespace: &mut N,
		generator: &mut G,
	) -> Self::TriplesAndValues<'a>
	where
		N::Id: 'a,
		L: 'a,
	{
		super::iter::Optional::new(
			self.as_ref()
				.map(|t| t.unbound_rdf_triples_and_values(namespace, generator)),
		)
	}
}

pub struct FlattenTriplesAndValues<I, U, L> {
	current: Option<Box<U>>,
	rest: I,
	_l: PhantomData<L>,
}

impl<'a, I: Iterator<Item = &'a T>, T: TriplesAndValues<N, L>, N: Namespace, L> RdfIterator<N>
	for FlattenTriplesAndValues<I, T::TriplesAndValues<'a>, L>
{
	type Item = TripleOrValue<N::Id, L>;

	fn next_with<G: Generator<N>>(
		&mut self,
		namespace: &mut N,
		generator: &mut G,
	) -> Option<Self::Item> {
		loop {
			match &mut self.current {
				Some(c) => match c.next_with(namespace, generator) {
					Some(item) => break Some(item),
					None => self.current = None,
				},
				None => match self.rest.next() {
					Some(i) => {
						self.current = Some(Box::new(
							i.unbound_rdf_triples_and_values(namespace, generator),
						))
					}
					None => break None,
				},
			}
		}
	}
}

impl<T: TriplesAndValues<N, L>, N: Namespace, L> TriplesAndValues<N, L> for BTreeSet<T> {
	type TriplesAndValues<'a> = FlattenTriplesAndValues<
		std::collections::btree_set::Iter<'a, T>,
		T::TriplesAndValues<'a>,
		L
	> where Self: 'a, N::Id: 'a, L: 'a;

	fn unbound_rdf_triples_and_values<'a, G: Generator<N>>(
		&'a self,
		_namespace: &mut N,
		_generator: &mut G,
	) -> Self::TriplesAndValues<'a>
	where
		N::Id: 'a,
		L: 'a,
	{
		FlattenTriplesAndValues {
			current: None,
			rest: self.iter(),
			_l: PhantomData,
		}
	}
}

/// RDF triples iterator provider.
///
/// The namespace `N` defines the node identifier type.
/// The type parameter `L` is the type of literal values.
pub trait Triples<N: Namespace, L> {
	/// Triples iterator.
	type Triples<'a>: 'a + RdfIterator<N, Item = Triple<N::Id, N::Id, Object<N::Id, L>>>
	where
		Self: 'a,
		N::Id: 'a,
		L: 'a;

	fn unbound_rdf_triples<'a, G: Generator<N>>(
		&'a self,
		namespace: &mut N,
		generator: &mut G,
	) -> Self::Triples<'_>
	where
		N::Id: 'a,
		L: 'a;

	fn rdf_triples<'a, 'n, 'g, G: Generator<N>>(
		&'a self,
		namespace: &'n mut N,
		generator: &'g mut G,
	) -> iter::Bound<'n, 'g, Self::Triples<'_>, N, G>
	where
		N::Id: 'a,
		L: 'a,
	{
		let inner = self.unbound_rdf_triples(namespace, generator);
		iter::Bound::new(inner, namespace, generator)
	}
}

impl<T: TriplesAndValues<N, L>, N: Namespace, L> Triples<N, L> for T {
	type Triples<'a> = FilterTriples<T::TriplesAndValues<'a>> where Self: 'a, N::Id: 'a, L: 'a;

	fn unbound_rdf_triples<'a, G: Generator<N>>(
		&'a self,
		namespace: &mut N,
		generator: &mut G,
	) -> Self::Triples<'a>
	where
		N::Id: 'a,
		L: 'a,
	{
		FilterTriples(self.unbound_rdf_triples_and_values(namespace, generator))
	}
}

/// Wrapper that changes a `TripleOrValue<I, L>` iterator into a
/// `Triple<I, I, Object<I, L>>` iterator.
pub struct FilterTriples<T>(pub T);

impl<N: Namespace, L, T: RdfIterator<N, Item = TripleOrValue<N::Id, L>>> RdfIterator<N>
	for FilterTriples<T>
{
	type Item = Triple<N::Id, N::Id, Object<N::Id, L>>;

	fn next_with<G: Generator<N>>(
		&mut self,
		namespace: &mut N,
		generator: &mut G,
	) -> Option<Self::Item> {
		loop {
			match self.0.next_with(namespace, generator) {
				Some(TripleOrValue::Triple(triple)) => break Some(triple),
				Some(TripleOrValue::Value(_)) => (),
				None => break None,
			}
		}
	}
}

pub struct LiteralValue<'a, T, I, L>(Option<&'a T>, PhantomData<(I, L)>);

impl<'a, T, I, L> LiteralValue<'a, T, I, L> {
	pub fn new(value: &'a T) -> Self {
		Self(Some(value), PhantomData)
	}
}

impl<'a, T: AsLiteral<N, L>, I, L, N: Namespace> RdfIterator<N> for LiteralValue<'a, T, I, L> {
	type Item = Object<I, L>;

	fn next_with<G: Generator<N>>(
		&mut self,
		namespace: &mut N,
		_generator: &mut G,
	) -> Option<Self::Item> {
		self.0
			.take()
			.map(|v| v.rdf_literal_value(namespace))
			.map(Object::Literal)
	}
}

pub struct IdValue<'a, I, L>(Option<&'a I>, PhantomData<L>);

impl<'a, I, L> IdValue<'a, I, L> {
	pub fn new(value: &'a I) -> Self {
		Self(Some(value), PhantomData)
	}
}

impl<'a, L, N: Namespace> RdfIterator<N> for IdValue<'a, N::Id, L>
where
	N::Id: Clone,
{
	type Item = Object<N::Id, L>;

	fn next_with<G: Generator<N>>(
		&mut self,
		_namespace: &mut N,
		_generator: &mut G,
	) -> Option<Self::Item> {
		self.0.take().cloned().map(Object::Id)
	}
}

pub struct ValuesOnly<T>(T);

impl<T> ValuesOnly<T> {
	pub fn new(inner: T) -> Self {
		ValuesOnly(inner)
	}
}

impl<L, T: RdfIterator<N, Item = Object<N::Id, L>>, N: Namespace> RdfIterator<N> for ValuesOnly<T> {
	type Item = TripleOrValue<N::Id, L>;

	fn next_with<G: Generator<N>>(
		&mut self,
		namespace: &mut N,
		generator: &mut G,
	) -> Option<Self::Item> {
		self.0
			.next_with(namespace, generator)
			.map(TripleOrValue::Value)
	}
}
