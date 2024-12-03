use std::{borrow::Cow, cell::RefCell, collections::HashMap, hash::Hash};

use iref::{Iri, IriBuf};
use rdf_types::{
	interpretation::{ReverseIriInterpretation, ReverseLiteralInterpretation},
	vocabulary::{ExtractFromVocabulary, IriVocabulary, LiteralVocabulary},
	Term,
};
use serde::{Deserialize, Serialize};
use serde_seeded::{DeserializeSeeded, SerializeSeeded};

use super::{RdfContext, RdfContextMut};

#[derive(Debug)]
pub struct ResourceForwardIndex<R>(HashMap<R, u32>);

impl<R> Default for ResourceForwardIndex<R> {
	fn default() -> Self {
		Self(HashMap::new())
	}
}

impl<R> ResourceForwardIndex<R> {
	pub fn new() -> Self {
		Self::default()
	}
}

impl<R> ResourceForwardIndex<R>
where
	R: Eq + Hash,
{
	pub fn get(&self, r: &R) -> Option<u32> {
		self.0.get(r).copied()
	}

	pub fn insert(&mut self, resource: R) -> u32 {
		let i = self.0.len() as u32;
		*self.0.entry(resource).or_insert(i)
	}
}

impl<C, R> SerializeSeeded<C> for ResourceForwardIndex<R>
where
	C: RdfContext<Resource = R>,
{
	fn serialize_seeded<S>(&self, rdf: &C, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let mut entries: Vec<(rdf_types::Term<&Iri, _>, u32)> = Vec::new();

		for (key, &value) in &self.0 {
			for i in rdf.interpretation().iris_of(key) {
				if let Some(iri) = rdf.vocabulary().iri(i) {
					entries.push((Term::Id(iri), value));
				}
			}

			for l in rdf.interpretation().literals_of(key) {
				if let Some(literal) = rdf.vocabulary().literal(l) {
					let literal = literal.extract_from_vocabulary(rdf.vocabulary());
					entries.push((Term::Literal(literal), value));
				}
			}
		}

		entries.serialize(serializer)
	}
}

#[derive(Debug)]
pub struct ResourceBackwardIndex<R>(HashMap<u32, R>);

impl<R> Default for ResourceBackwardIndex<R> {
	fn default() -> Self {
		Self(HashMap::new())
	}
}

impl<R> ResourceBackwardIndex<R> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn get(&self, i: u32) -> Option<&R> {
		self.0.get(&i)
	}

	pub fn insert(&mut self, i: u32, resource: R) {
		self.0.insert(i, resource);
	}

	// pub fn deserialize_seed<T>(&self) -> ResourceBackwardIndexSeed<R, T> {
	// 	ResourceBackwardIndexSeed::new(self)
	// }
}

impl<'de, C, R> DeserializeSeeded<'de, RefCell<C>> for ResourceBackwardIndex<R>
where
	C: RdfContextMut<Resource = R>,
{
	fn deserialize_seeded<D>(rdf: &RefCell<C>, deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let entries: Vec<(Term<IriBuf, rdf_types::Literal>, u32)> = Vec::deserialize(deserializer)?;

		let mut rdf = rdf.borrow_mut();
		let mut result = Self::new();
		for (key, value) in entries {
			let resource = match key {
				Term::Id(iri) => rdf.insert_iri(Cow::Owned(iri)),
				Term::Literal(literal) => match literal.type_ {
					rdf_types::LiteralType::Any(iri) => {
						rdf.insert_literal(literal.value, Cow::Owned(iri))
					}
					rdf_types::LiteralType::LangString(tag) => {
						rdf.insert_lang_string(literal.value, tag)
					}
				},
			};

			use std::collections::hash_map::Entry;
			match result.0.entry(value) {
				Entry::Occupied(e) => {
					rdf.unify(&resource, e.get())
						.map_err(|_| serde::de::Error::custom("resource unification failed"))?;
				}
				Entry::Vacant(e) => {
					e.insert(resource);
				}
			}
		}

		Ok(result)
	}
}
