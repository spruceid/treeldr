use rdf_types::Interpretation;

/// Iterator that borrows the RDF namespace and name generator
pub trait RdfIterator<V, I: Interpretation> {
	type Item;

	fn next_with(
		&mut self,
		vocabulary: &mut V,
		interpretation: &mut I,
		graph: Option<&I::Resource>,
	) -> Option<Self::Item>;
}

pub trait IntoRdfIterator<V, I: Interpretation> {
	type Item;
	type IntoIter: RdfIterator<V, I, Item = Self::Item>;

	fn into_rdf_iter(self, vocabulary: &mut V, interpretation: &mut I) -> Option<Self::IntoIter>;
}

/// Iterator bound to a namespace and generator.
pub struct Bound<'v, 'i, 'g, T, V, I: Interpretation> {
	inner: T,
	vocabulary: &'v mut V,
	interpretation: &'i mut I,
	graph: Option<&'g I::Resource>,
}

impl<'v, 'i, 'g, T, V, I: Interpretation> Bound<'v, 'i, 'g, T, V, I> {
	pub fn new(
		inner: T,
		vocabulary: &'v mut V,
		interpretation: &'i mut I,
		graph: Option<&'g I::Resource>,
	) -> Self {
		Self {
			inner,
			vocabulary,
			interpretation,
			graph,
		}
	}

	pub fn interpretation(&self) -> &I {
		self.interpretation
	}

	pub fn interpretation_mut(&mut self) -> &mut I {
		self.interpretation
	}
}

impl<'v, 'i, 'g, T: RdfIterator<V, I>, V, I: Interpretation> Iterator
	for Bound<'v, 'i, 'g, T, V, I>
{
	type Item = T::Item;

	fn next(&mut self) -> Option<Self::Item> {
		self.inner
			.next_with(self.vocabulary, self.interpretation, self.graph)
	}
}

pub struct Once<T>(Option<T>);

impl<T, V, I: Interpretation> RdfIterator<V, I> for Once<T> {
	type Item = T;

	fn next_with(
		&mut self,
		_vocabulary: &mut V,
		_interpretation: &mut I,
		_graph: Option<&I::Resource>,
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

impl<T: RdfIterator<V, I>, V, I: Interpretation> RdfIterator<V, I> for Optional<T> {
	type Item = T::Item;

	fn next_with(
		&mut self,
		vocabulary: &mut V,
		interpretation: &mut I,
		graph: Option<&I::Resource>,
	) -> Option<Self::Item> {
		self.0
			.as_mut()
			.and_then(|inner| inner.next_with(vocabulary, interpretation, graph))
	}
}
