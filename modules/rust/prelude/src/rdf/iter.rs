use rdf_types::{Generator, Namespace};

/// Iterator that borrows the RDF namespace and name generator
pub trait RdfIterator<N: Namespace> {
	type Item;

	fn next_with<G: Generator<N>>(
		&mut self,
		namespace: &mut N,
		generator: &mut G,
	) -> Option<Self::Item>;
}

pub trait IntoRdfIterator<N: Namespace> {
	type Item;
	type IntoIter: RdfIterator<N, Item = Self::Item>;

	fn next_with<G: Generator<N>>(
		&mut self,
		namespace: &mut N,
		generator: &mut G,
	) -> Option<Self::IntoIter>;
}

/// Iterator bound to a namespace and generator.
pub struct Bound<'n, 'g, I, N, G> {
	inner: I,
	namespace: &'n mut N,
	generator: &'g mut G,
}

impl<'n, 'g, I, N, G> Bound<'n, 'g, I, N, G> {
	pub fn new(inner: I, namespace: &'n mut N, generator: &'g mut G) -> Self {
		Self {
			inner,
			namespace,
			generator,
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

	fn next_with<G: Generator<N>>(
		&mut self,
		_namespace: &mut N,
		_generator: &mut G,
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

	fn next_with<G: Generator<N>>(
		&mut self,
		namespace: &mut N,
		generator: &mut G,
	) -> Option<Self::Item> {
		self.0
			.as_mut()
			.and_then(|inner| inner.next_with(namespace, generator))
	}
}
