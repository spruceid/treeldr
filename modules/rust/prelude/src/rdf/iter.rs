use std::marker::PhantomData;

use rdf_types::{Namespace, Generator, Object};

use super::{TripleOrValue, AsLiteral};

/// Iterator that borrows the RDF namespace and name generator
pub trait RdfIterator<N: Namespace> {
	type Item;

	fn next_with<'n, 'g, G: Generator<N>>(
		&mut self,
		namespace: &'n mut N,
		generator: &'g mut G
	) -> Option<Self::Item>;
}

/// Iterator bound to a namespace and generator.
pub struct Bound<'n, 'g, I, N, G> {
	inner: I,
	namespace: &'n mut N,
	generator: &'g mut G
}

impl<'n, 'g, I, N, G> Bound<'n, 'g, I, N, G> {
	pub fn new(
		inner: I,
		namespace: &'n mut N,
		generator: &'g mut G
	) -> Self {
		Self {
			inner,
			namespace,
			generator
		}
	}
}

impl<'n, 'g, I: RdfIterator<N>, N: Namespace, G: Generator<N>> Iterator for Bound<'n, 'g, I, N, G> {
	type Item = I::Item;

	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next_with(self.namespace, self.generator)
	}
}

pub struct Once<T>(Option<T>);

impl<T, N: Namespace> RdfIterator<N> for Once<T> {
	type Item = T;

	fn next_with<'n, 'g, G: Generator<N>>(
		&mut self,
		_namespace: &'n mut N,
		_generator: &'g mut G
	) -> Option<Self::Item> {
		self.0.take()
	}
}

pub struct Optional<T>(Option<T>);

impl<T> Optional<T> {
	pub fn new(inner: Option<T>) -> Self {
		Self(inner)
	}
}

impl<T: RdfIterator<N>, N: Namespace> RdfIterator<N> for Optional<T> {
	type Item = T::Item;

	fn next_with<'n, 'g, G: Generator<N>>(
		&mut self,
		namespace: &'n mut N,
		generator: &'g mut G
	) -> Option<Self::Item> {
		self.0.as_mut().and_then(|inner| inner.next_with(namespace, generator))
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

	fn next_with<'n, 'g, G: Generator<N>>(
		&mut self,
		namespace: &'n mut N,
		_generator: &'g mut G
	) -> Option<Self::Item> {
		self.0.take().map(|v| v.rdf_literal_value(namespace)).map(Object::Literal)
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

	fn next_with<'n, 'g, G: Generator<N>>(
			&mut self,
			namespace: &'n mut N,
			generator: &'g mut G
		) -> Option<Self::Item> {
		self.0.next_with(namespace, generator).map(TripleOrValue::Value)
	}
}