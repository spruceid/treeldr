use std::{collections::BTreeSet, marker::PhantomData};

use rdf_types::{Generator, Namespace, Object, Quad};

use crate::{Id, RdfIterator};

mod literal;

pub use literal::*;

use super::iter;

/// Quad or value sum type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum QuadOrValue<I, L> {
	Quad(Quad<I, I, Object<I, L>, I>),
	Value(Object<I, L>),
}

/// RDF traversal.
pub trait QuadsAndValues<N: Namespace, L> {
	type QuadsAndValues<'a>: 'a + RdfIterator<N, Item = QuadOrValue<N::Id, L>>
	where
		Self: 'a,
		N::Id: 'a,
		L: 'a;

	fn unbound_rdf_quads_and_values<'a, G: Generator<N>>(
		&'a self,
		namespace: &mut N,
		generator: &mut G,
	) -> Self::QuadsAndValues<'a>
	where
		N::Id: 'a,
		L: 'a;

	fn rdf_triples_and_values<'a, 'n, 'g, 't, G: Generator<N>>(
		&'a self,
		namespace: &'n mut N,
		generator: &'g mut G,
		graph: Option<&'t N::Id>,
	) -> iter::Bound<'n, 'g, 't, Self::QuadsAndValues<'a>, N, G>
	where
		N::Id: 'a,
		L: 'a,
	{
		let inner = self.unbound_rdf_quads_and_values(namespace, generator);
		iter::Bound::new(inner, namespace, generator, graph)
	}
}

impl<N: Namespace, L> QuadsAndValues<N, L> for Id<N::Id>
where
	N::Id: Clone,
{
	type QuadsAndValues<'a> = ValuesOnly<IdValue<'a, N::Id, L>> where Self: 'a, N::Id: 'a, L: 'a;

	fn unbound_rdf_quads_and_values<'a, G: Generator<N>>(
		&'a self,
		_namespace: &mut N,
		_generator: &mut G,
	) -> Self::QuadsAndValues<'a>
	where
		N::Id: 'a,
		L: 'a,
	{
		ValuesOnly::new(IdValue::new(&self.0))
	}
}

impl<T: QuadsAndValues<N, L>, N: Namespace, L> QuadsAndValues<N, L> for Option<T> {
	type QuadsAndValues<'a> = super::iter::Optional<T::QuadsAndValues<'a>> where Self: 'a, N::Id: 'a, L: 'a;

	fn unbound_rdf_quads_and_values<'a, G: Generator<N>>(
		&'a self,
		namespace: &mut N,
		generator: &mut G,
	) -> Self::QuadsAndValues<'a>
	where
		N::Id: 'a,
		L: 'a,
	{
		super::iter::Optional::new(
			self.as_ref()
				.map(|t| t.unbound_rdf_quads_and_values(namespace, generator)),
		)
	}
}

pub struct FlattenQuadsAndValues<I, U, L> {
	current: Option<Box<U>>,
	rest: I,
	_l: PhantomData<L>,
}

impl<'a, I: Iterator<Item = &'a T>, T: QuadsAndValues<N, L>, N: Namespace, L> RdfIterator<N>
	for FlattenQuadsAndValues<I, T::QuadsAndValues<'a>, L>
{
	type Item = QuadOrValue<N::Id, L>;

	fn next_with<G: Generator<N>>(
		&mut self,
		namespace: &mut N,
		generator: &mut G,
		graph: Option<&N::Id>,
	) -> Option<Self::Item> {
		loop {
			match &mut self.current {
				Some(c) => match c.next_with(namespace, generator, graph) {
					Some(item) => break Some(item),
					None => self.current = None,
				},
				None => match self.rest.next() {
					Some(i) => {
						self.current = Some(Box::new(
							i.unbound_rdf_quads_and_values(namespace, generator),
						))
					}
					None => break None,
				},
			}
		}
	}
}

impl<T: QuadsAndValues<N, L>, N: Namespace, L> QuadsAndValues<N, L> for BTreeSet<T> {
	type QuadsAndValues<'a> = FlattenQuadsAndValues<
		std::collections::btree_set::Iter<'a, T>,
		T::QuadsAndValues<'a>,
		L
	> where Self: 'a, N::Id: 'a, L: 'a;

	fn unbound_rdf_quads_and_values<'a, G: Generator<N>>(
		&'a self,
		_namespace: &mut N,
		_generator: &mut G,
	) -> Self::QuadsAndValues<'a>
	where
		N::Id: 'a,
		L: 'a,
	{
		FlattenQuadsAndValues {
			current: None,
			rest: self.iter(),
			_l: PhantomData,
		}
	}
}

/// RDF quads iterator provider.
///
/// The namespace `N` defines the node identifier type.
/// The type parameter `L` is the type of literal values.
pub trait Quads<N: Namespace, L> {
	/// Triples iterator.
	type Quads<'a>: 'a + RdfIterator<N, Item = Quad<N::Id, N::Id, Object<N::Id, L>, N::Id>>
	where
		Self: 'a,
		N::Id: 'a,
		L: 'a;

	fn unbound_rdf_quads<'a, G: Generator<N>>(
		&'a self,
		namespace: &mut N,
		generator: &mut G,
	) -> Self::Quads<'_>
	where
		N::Id: 'a,
		L: 'a;

	fn rdf_quads<'a, 'n, 'g, 't, G: Generator<N>>(
		&'a self,
		namespace: &'n mut N,
		generator: &'g mut G,
		graph: Option<&'t N::Id>,
	) -> iter::Bound<'n, 'g, 't, Self::Quads<'_>, N, G>
	where
		N::Id: 'a,
		L: 'a,
	{
		let inner = self.unbound_rdf_quads(namespace, generator);
		iter::Bound::new(inner, namespace, generator, graph)
	}
}

impl<T: QuadsAndValues<N, L>, N: Namespace, L> Quads<N, L> for T {
	type Quads<'a> = FilterQuads<T::QuadsAndValues<'a>> where Self: 'a, N::Id: 'a, L: 'a;

	fn unbound_rdf_quads<'a, G: Generator<N>>(
		&'a self,
		namespace: &mut N,
		generator: &mut G,
	) -> Self::Quads<'a>
	where
		N::Id: 'a,
		L: 'a,
	{
		FilterQuads(self.unbound_rdf_quads_and_values(namespace, generator))
	}
}

/// Wrapper that changes a `TripleOrValue<I, L>` iterator into a
/// `Triple<I, I, Object<I, L>>` iterator.
pub struct FilterQuads<T>(pub T);

impl<N: Namespace, L, T: RdfIterator<N, Item = QuadOrValue<N::Id, L>>> RdfIterator<N>
	for FilterQuads<T>
{
	type Item = Quad<N::Id, N::Id, Object<N::Id, L>, N::Id>;

	fn next_with<G: Generator<N>>(
		&mut self,
		namespace: &mut N,
		generator: &mut G,
		graph: Option<&N::Id>,
	) -> Option<Self::Item> {
		loop {
			match self.0.next_with(namespace, generator, graph) {
				Some(QuadOrValue::Quad(triple)) => break Some(triple),
				Some(QuadOrValue::Value(_)) => (),
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
		_graph: Option<&N::Id>,
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
		_graph: Option<&N::Id>,
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
	type Item = QuadOrValue<N::Id, L>;

	fn next_with<G: Generator<N>>(
		&mut self,
		namespace: &mut N,
		generator: &mut G,
		graph: Option<&N::Id>,
	) -> Option<Self::Item> {
		self.0
			.next_with(namespace, generator, graph)
			.map(QuadOrValue::Value)
	}
}
