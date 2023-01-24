use std::collections::HashMap;

use iref::Iri;
use nquads_syntax::BlankIdBuf;
use rdf_types::{
	BlankId, BlankIdVocabulary, BlankIdVocabularyMut, Generator, IriVocabulary, IriVocabularyMut,
	VocabularyMut,
};
use treeldr::{BlankIdIndex, Id, IriIndex};

pub struct ScopedVocabulary<'a, V, S> {
	scope: S,
	map: HashMap<BlankIdBuf, BlankIdIndex>,
	inner: &'a mut V,
}

impl<'a, V, S> ScopedVocabulary<'a, V, S> {
	pub fn new(vocabulary: &'a mut V, scope: S) -> Self {
		Self {
			scope,
			map: HashMap::new(),
			inner: vocabulary,
		}
	}
}

impl<'a, V: IriVocabulary<Iri = IriIndex>, S> IriVocabulary for ScopedVocabulary<'a, V, S> {
	type Iri = IriIndex;

	fn iri<'i>(&'i self, id: &'i Self::Iri) -> Option<Iri<'i>> {
		self.inner.iri(id)
	}

	fn get(&self, iri: Iri) -> Option<Self::Iri> {
		self.inner.get(iri)
	}
}

impl<'a, V: IriVocabularyMut<Iri = IriIndex>, S> IriVocabularyMut for ScopedVocabulary<'a, V, S> {
	fn insert(&mut self, iri: Iri) -> Self::Iri {
		self.inner.insert(iri)
	}
}

impl<'a, V: BlankIdVocabulary<BlankId = BlankIdIndex>, S> BlankIdVocabulary
	for ScopedVocabulary<'a, V, S>
{
	type BlankId = BlankIdIndex;

	fn blank_id<'b>(&'b self, id: &'b Self::BlankId) -> Option<&'b BlankId> {
		self.inner.blank_id(id)
	}

	fn get_blank_id(&self, id: &BlankId) -> Option<Self::BlankId> {
		self.map.get(id).copied()
	}
}

impl<'a, V: BlankIdVocabularyMut<BlankId = BlankIdIndex>, S: std::fmt::Display> BlankIdVocabularyMut
	for ScopedVocabulary<'a, V, S>
{
	fn insert_blank_id(&mut self, id: &BlankId) -> Self::BlankId {
		match self.get_blank_id(id) {
			Some(id) => id,
			None => {
				let scoped =
					BlankIdBuf::from_suffix(&format!("{}:{}", self.scope, id.suffix())).unwrap();
				let i = self.inner.insert_blank_id(&scoped);
				self.map.insert(id.to_owned(), i);
				i
			}
		}
	}
}

pub struct ScopedGenerator<'a, G>(pub &'a mut G);

impl<'a, 'v, G: Generator<V>, V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>, S>
	Generator<ScopedVocabulary<'v, V, S>> for ScopedGenerator<'a, G>
{
	fn next(&mut self, vocabulary: &mut ScopedVocabulary<'v, V, S>) -> Id {
		self.0.next(vocabulary.inner)
	}
}
