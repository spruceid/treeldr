use std::collections::BTreeSet;

use rdf_types::{Interpretation, LiteralInterpretationMut, LiteralVocabulary, Quad};

use crate::RdfIterator;

mod literal;

pub use literal::*;

use super::iter;

/// Quad or value sum type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum QuadOrValue<T> {
	Quad(Quad<T, T, T, T>),
	Value(T),
}

/// RDF traversal.
pub trait QuadsAndValues<V, I: Interpretation> {
	type QuadsAndValues<'a>: 'a + RdfIterator<V, I, Item = QuadOrValue<I::Resource>>
	where
		Self: 'a,
		I::Resource: 'a;

	fn unbound_rdf_quads_and_values<'a>(
		&'a self,
		vocabulary: &mut V,
		interpretation: &mut I,
	) -> (Option<I::Resource>, Self::QuadsAndValues<'a>)
	where
		I::Resource: 'a;

	fn rdf_triples_and_values<'a, 'v, 'i, 't>(
		&'a self,
		vocabulary: &'v mut V,
		interpretation: &'i mut I,
		graph: Option<&'t I::Resource>,
	) -> (
		Option<I::Resource>,
		iter::Bound<'v, 'i, 't, Self::QuadsAndValues<'a>, V, I>,
	)
	where
		I::Resource: 'a,
	{
		let (id, inner) = self.unbound_rdf_quads_and_values(vocabulary, interpretation);
		(
			id,
			iter::Bound::new(inner, vocabulary, interpretation, graph),
		)
	}
}

impl<'t, V, T: QuadsAndValues<V, I>, I: Interpretation> QuadsAndValues<V, I> for &'t T {
	type QuadsAndValues<'a> = T::QuadsAndValues<'a> where Self: 'a, I::Resource: 'a;

	fn unbound_rdf_quads_and_values<'a>(
		&'a self,
		vocabulary: &mut V,
		interpretation: &mut I,
	) -> (Option<I::Resource>, Self::QuadsAndValues<'a>)
	where
		I::Resource: 'a,
	{
		T::unbound_rdf_quads_and_values(self, vocabulary, interpretation)
	}
}

impl<T: QuadsAndValues<V, I>, V, I: Interpretation> QuadsAndValues<V, I> for Option<T> {
	type QuadsAndValues<'a> = super::iter::Optional<T::QuadsAndValues<'a>> where Self: 'a, I::Resource: 'a;

	fn unbound_rdf_quads_and_values<'a>(
		&'a self,
		vocabulary: &mut V,
		interpretation: &mut I,
	) -> (Option<I::Resource>, Self::QuadsAndValues<'a>)
	where
		I::Resource: 'a,
	{
		match self.as_ref() {
			Some(t) => {
				let (id, inner) = t.unbound_rdf_quads_and_values(vocabulary, interpretation);
				(id, super::iter::Optional::new(Some(inner)))
			}
			None => (None, super::iter::Optional::new(None)),
		}
	}
}

pub struct FlattenQuadsAndValues<I, U> {
	current: Option<Box<U>>,
	rest: I,
}

impl<'a, I: Iterator<Item = &'a T>, T: QuadsAndValues<V, N>, V, N: Interpretation> RdfIterator<V, N>
	for FlattenQuadsAndValues<I, T::QuadsAndValues<'a>>
{
	type Item = QuadOrValue<N::Resource>;

	fn next_with(
		&mut self,
		vocabulary: &mut V,
		interpretation: &mut N,
		graph: Option<&N::Resource>,
	) -> Option<Self::Item> {
		loop {
			match &mut self.current {
				Some(c) => match c.next_with(vocabulary, interpretation, graph) {
					Some(item) => break Some(item),
					None => self.current = None,
				},
				None => match self.rest.next() {
					Some(i) => {
						self.current = Some(Box::new(
							i.unbound_rdf_quads_and_values(vocabulary, interpretation).1,
						))
					}
					None => break None,
				},
			}
		}
	}
}

impl<T: QuadsAndValues<V, I>, V, I: Interpretation> QuadsAndValues<V, I> for BTreeSet<T> {
	type QuadsAndValues<'a> = FlattenQuadsAndValues<
		std::collections::btree_set::Iter<'a, T>,
		T::QuadsAndValues<'a>
	> where Self: 'a, I::Resource: 'a;

	fn unbound_rdf_quads_and_values<'a>(
		&'a self,
		_vocabulary: &mut V,
		_interpretation: &mut I,
	) -> (Option<I::Resource>, Self::QuadsAndValues<'a>)
	where
		I::Resource: 'a,
	{
		(
			None,
			FlattenQuadsAndValues {
				current: None,
				rest: self.iter(),
			},
		)
	}
}

/// RDF quads iterator provider.
///
/// The namespace `N` defines the node identifier type.
/// The type parameter `L` is the type of literal values.
pub trait Quads<V, I: Interpretation> {
	/// Triples iterator.
	type Quads<'a>: 'a
		+ RdfIterator<V, I, Item = Quad<I::Resource, I::Resource, I::Resource, I::Resource>>
	where
		Self: 'a,
		I::Resource: 'a;

	fn unbound_rdf_quads<'a>(
		&'a self,
		vocabulary: &mut V,
		interpretation: &mut I,
	) -> (Option<I::Resource>, Self::Quads<'_>)
	where
		I::Resource: 'a;

	fn rdf_quads<'a, 'v, 'i, 'g>(
		&'a self,
		vocabulary: &'v mut V,
		interpretation: &'i mut I,
		graph: Option<&'g I::Resource>,
	) -> (
		Option<I::Resource>,
		iter::Bound<'v, 'i, 'g, Self::Quads<'_>, V, I>,
	)
	where
		I::Resource: 'a,
	{
		let (id, inner) = self.unbound_rdf_quads(vocabulary, interpretation);
		(
			id,
			iter::Bound::new(inner, vocabulary, interpretation, graph),
		)
	}
}

impl<T: QuadsAndValues<V, I>, V, I: Interpretation> Quads<V, I> for T {
	type Quads<'a> = FilterQuads<T::QuadsAndValues<'a>> where Self: 'a, I::Resource: 'a;

	fn unbound_rdf_quads<'a>(
		&'a self,
		vocabulary: &mut V,
		interpretation: &mut I,
	) -> (Option<I::Resource>, Self::Quads<'a>)
	where
		I::Resource: 'a,
	{
		let (id, quads_and_values) = self.unbound_rdf_quads_and_values(vocabulary, interpretation);
		(id, FilterQuads(quads_and_values))
	}
}

/// Wrapper that changes a `QuadOrValue<I>` iterator into a
/// `Quad<I, I, I, I>` iterator.
pub struct FilterQuads<T>(pub T);

impl<V, I: Interpretation, T: RdfIterator<V, I, Item = QuadOrValue<I::Resource>>> RdfIterator<V, I>
	for FilterQuads<T>
{
	type Item = Quad<I::Resource, I::Resource, I::Resource, I::Resource>;

	fn next_with(
		&mut self,
		vocabulary: &mut V,
		interpretation: &mut I,
		graph: Option<&I::Resource>,
	) -> Option<Self::Item> {
		loop {
			match self.0.next_with(vocabulary, interpretation, graph) {
				Some(QuadOrValue::Quad(triple)) => break Some(triple),
				Some(QuadOrValue::Value(_)) => (),
				None => break None,
			}
		}
	}
}

pub struct LiteralValue<'a, T>(Option<&'a T>);

impl<'a, T> LiteralValue<'a, T> {
	pub fn new(value: &'a T) -> Self {
		Self(Some(value))
	}
}

impl<'a, T: AsLiteral<V>, V: LiteralVocabulary, I: LiteralInterpretationMut<V::Literal>>
	RdfIterator<V, I> for LiteralValue<'a, T>
{
	type Item = I::Resource;

	fn next_with(
		&mut self,
		vocabulary: &mut V,
		interpretation: &mut I,
		_graph: Option<&I::Resource>,
	) -> Option<Self::Item> {
		self.0
			.take()
			.map(|v| v.rdf_literal_value(vocabulary))
			.map(|l| interpretation.interpret_literal(l))
	}
}

impl<V, I: Interpretation> QuadsAndValues<V, I> for crate::Id<I::Resource>
where
	I::Resource: Clone,
{
	type QuadsAndValues<'a> = ValuesOnly<IdValue<'a, I::Resource>> where Self: 'a;

	fn unbound_rdf_quads_and_values<'a>(
		&'a self,
		_vocabulary: &mut V,
		_interpretation: &mut I,
	) -> (
		Option<<I as Interpretation>::Resource>,
		Self::QuadsAndValues<'a>,
	)
	where
		<I as Interpretation>::Resource: 'a,
	{
		(Some(self.0.clone()), ValuesOnly(IdValue(Some(&self.0))))
	}
}

pub struct IdValue<'a, T>(Option<&'a T>);

impl<'a, T> IdValue<'a, T> {
	pub fn new(value: &'a T) -> Self {
		Self(Some(value))
	}
}

impl<'a, V, I: Interpretation> RdfIterator<V, I> for IdValue<'a, I::Resource>
where
	I::Resource: Clone,
{
	type Item = I::Resource;

	fn next_with(
		&mut self,
		_vocabulary: &mut V,
		_interpretation: &mut I,
		_graph: Option<&I::Resource>,
	) -> Option<Self::Item> {
		self.0.take().cloned()
	}
}

pub struct ValuesOnly<T>(T);

impl<T> ValuesOnly<T> {
	pub fn new(inner: T) -> Self {
		ValuesOnly(inner)
	}
}

impl<T: RdfIterator<V, I, Item = I::Resource>, V, I: Interpretation> RdfIterator<V, I>
	for ValuesOnly<T>
{
	type Item = QuadOrValue<I::Resource>;

	fn next_with(
		&mut self,
		vocabulary: &mut V,
		interpretation: &mut I,
		graph: Option<&I::Resource>,
	) -> Option<Self::Item> {
		self.0
			.next_with(vocabulary, interpretation, graph)
			.map(QuadOrValue::Value)
	}
}
