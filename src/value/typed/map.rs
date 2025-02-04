use std::{cmp::Ordering, fmt, hash::Hash, marker::PhantomData};

use raw_btree::RawBTree;
use serde::ser::SerializeMap;
use serde_seeded::{de::Seed, ser::Seeded, DeserializeSeeded, SerializeSeeded};

use crate::{rdf::VisitResources, value::ValueLike, TypeRef};

use super::TypedValue;

#[derive(Clone)]
pub struct TypedValueMap<P, R, V, T = TypeRef<P>>(RawBTree<Entry<P, R, V, T>>);

impl<P, R, V, T> TypedValueMap<P, R, V, T> {
	pub fn new() -> Self {
		Self(RawBTree::new())
	}

	pub fn len(&self) -> usize {
		self.0.len()
	}

	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	pub fn iter(&self) -> Iter<P, R, V, T> {
		Iter(self.0.iter())
	}
}

impl<P, R, V, T> Default for TypedValueMap<P, R, V, T> {
	fn default() -> Self {
		Self::new()
	}
}

impl<P, R: Ord, V, T> TypedValueMap<P, R, V, T> {
	pub fn get_entry_untyped(
		&self,
		key: &impl ValueLike<Resource = R>,
	) -> Option<(&TypedValue<P, R, T>, &V)> {
		self.0.get(Entry::untyped_cmp, key).map(Entry::as_pair)
	}

	pub fn get_untyped(&self, key: &impl ValueLike<Resource = R>) -> Option<&V> {
		Some(self.get_entry_untyped(key)?.1)
	}

	pub fn insert(
		&mut self,
		key: TypedValue<P, R, T>,
		value: V,
	) -> Option<(TypedValue<P, R, T>, V)> {
		self.0
			.insert(Entry::untyped_entry_cmp, Entry::new(key, value))
			.map(Entry::into_pair)
	}

	// pub fn pattern_matching<'a>(&'a self, pattern: &'a TypedPattern<R>) -> impl 'a + Iterator<Item = (&TypedValue<P, R, T>, &V)> {
	// 	self.iter().filter(|(key, _)| pattern.matches(key))
	// }
}

impl<P, R, T, V: VisitResources<R>> VisitResources<R> for TypedValueMap<P, R, V, T> {
	fn visit_resources_ref_mut(&self, f: &mut impl FnMut(&R)) {
		for (key, value) in self {
			key.visit_resources_ref_mut(f);
			value.visit_resources_ref_mut(f)
		}
	}
}

impl<P: PartialEq, R: Ord, V, T: PartialEq> TypedValueMap<P, R, V, T> {
	pub fn get_entry(&self, key: &TypedValue<P, R, T>) -> Option<(&TypedValue<P, R, T>, &V)> {
		self.get_entry_untyped(key)
			.filter(|(entry_key, _)| *entry_key == key)
	}

	pub fn get(&self, key: &TypedValue<P, R, T>) -> Option<&V> {
		Some(self.get_entry(key)?.1)
	}
}

impl<P: fmt::Debug, R: fmt::Debug, V: fmt::Debug, T: fmt::Debug> fmt::Debug
	for TypedValueMap<P, R, V, T>
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_map().entries(self.iter()).finish()
	}
}

impl<P: PartialEq, R: PartialEq, V: PartialEq, T: PartialEq> PartialEq
	for TypedValueMap<P, R, V, T>
{
	fn eq(&self, other: &Self) -> bool {
		self.iter().eq(other.iter())
	}
}

impl<P: Eq, R: Eq, V: Eq, T: Eq> Eq for TypedValueMap<P, R, V, T> {}

impl<P: PartialOrd, R: PartialOrd, V: PartialOrd, T: PartialOrd> PartialOrd
	for TypedValueMap<P, R, V, T>
{
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		self.iter().partial_cmp(other.iter())
	}
}

impl<P: Ord, R: Ord, V: Ord, T: Ord> Ord for TypedValueMap<P, R, V, T> {
	fn cmp(&self, other: &Self) -> Ordering {
		self.iter().cmp(other.iter())
	}
}

impl<P: Hash, R: Hash, V: Hash, T: Hash> Hash for TypedValueMap<P, R, V, T> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		for entry in &self.0 {
			entry.hash(state);
		}
	}
}

impl<P, R: Ord, V, T: Ord> Extend<(TypedValue<P, R, T>, V)> for TypedValueMap<P, R, V, T> {
	fn extend<I: IntoIterator<Item = (TypedValue<P, R, T>, V)>>(&mut self, iter: I) {
		for (key, value) in iter {
			self.insert(key, value);
		}
	}
}

impl<P, R: Ord, V, T: Ord> FromIterator<(TypedValue<P, R, T>, V)> for TypedValueMap<P, R, V, T> {
	fn from_iter<I: IntoIterator<Item = (TypedValue<P, R, T>, V)>>(iter: I) -> Self {
		let mut result = Self::default();
		result.extend(iter);
		result
	}
}

pub struct Iter<'a, P, R, V, T>(raw_btree::Iter<'a, Entry<P, R, V, T>>);

impl<'a, P, R, V, T> Iterator for Iter<'a, P, R, V, T> {
	type Item = (&'a TypedValue<P, R, T>, &'a V);

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next().map(Entry::as_pair)
	}
}

impl<'a, P, R, V, T> IntoIterator for &'a TypedValueMap<P, R, V, T> {
	type Item = (&'a TypedValue<P, R, T>, &'a V);
	type IntoIter = Iter<'a, P, R, V, T>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

pub struct IntoIter<P, R, V, T>(raw_btree::IntoIter<Entry<P, R, V, T>>);

impl<P, R, V, T> Iterator for IntoIter<P, R, V, T> {
	type Item = (TypedValue<P, R, T>, V);

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next().map(Entry::into_pair)
	}
}

impl<P, R, V, T> IntoIterator for TypedValueMap<P, R, V, T> {
	type Item = (TypedValue<P, R, T>, V);
	type IntoIter = IntoIter<P, R, V, T>;

	fn into_iter(self) -> Self::IntoIter {
		IntoIter(self.0.into_iter())
	}
}

#[derive(Clone, Hash)]
struct Entry<P, R, V, T> {
	key: TypedValue<P, R, T>,
	value: V,
}

impl<P, R, V, T> Entry<P, R, V, T> {
	fn new(key: TypedValue<P, R, T>, value: V) -> Self {
		Self { key, value }
	}

	fn untyped_cmp(&self, value: &impl ValueLike<Resource = R>) -> Ordering
	where
		R: Ord,
	{
		ValueLike::cmp(&self.key, value)
	}

	fn untyped_entry_cmp(&self, other: &Self) -> Ordering
	where
		R: Ord,
	{
		ValueLike::cmp(&self.key, &other.key)
	}

	fn as_pair(&self) -> (&TypedValue<P, R, T>, &V) {
		(&self.key, &self.value)
	}

	fn into_pair(self) -> (TypedValue<P, R, T>, V) {
		(self.key, self.value)
	}
}

impl<Q, P, R, V, T> SerializeSeeded<Q> for TypedValueMap<P, R, V, T>
where
	TypedValue<P, R, T>: SerializeSeeded<Q>,
	V: SerializeSeeded<Q>,
{
	fn serialize_seeded<S>(&self, seed: &Q, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let mut map = serializer.serialize_map(Some(self.len()))?;

		for (key, value) in self {
			map.serialize_entry(&Seeded::new(seed, key), &Seeded::new(seed, value))?
		}

		map.end()
	}
}

impl<'de, Q, P, R, V, T> DeserializeSeeded<'de, Q> for TypedValueMap<P, R, V, T>
where
	R: Ord,
	TypedValue<P, R, T>: DeserializeSeeded<'de, Q>,
	V: DeserializeSeeded<'de, Q>,
{
	fn deserialize_seeded<D>(seed: &Q, deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		struct Visitor<'seed, Q, P, R, V, T> {
			seed: &'seed Q,
			p: PhantomData<(P, R, V, T)>,
		}

		impl<'de, Q, P, R, V, T> serde::de::Visitor<'de> for Visitor<'_, Q, P, R, V, T>
		where
			R: Ord,
			TypedValue<P, R, T>: DeserializeSeeded<'de, Q>,
			V: DeserializeSeeded<'de, Q>,
		{
			type Value = TypedValueMap<P, R, V, T>;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				write!(formatter, "a typed value map")
			}

			fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
			where
				A: serde::de::MapAccess<'de>,
			{
				let mut result = TypedValueMap::new();

				while let Some((key, value)) =
					map.next_entry_seed(Seed::new(self.seed), Seed::new(self.seed))?
				{
					result.insert(key, value);
				}

				Ok(result)
			}
		}

		deserializer.deserialize_map(Visitor {
			seed,
			p: PhantomData,
		})
	}
}
