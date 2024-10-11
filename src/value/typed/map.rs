use std::{cmp::Ordering, fmt, hash::Hash};

use raw_btree::RawBTree;

use crate::{value::ValueLike, TypeRef};

use super::TypedValue;

#[derive(Clone)]
pub struct TypedMap<R, V, T = TypeRef<R>>(RawBTree<Entry<R, V, T>>);

impl<R, V, T> TypedMap<R, V, T> {
	pub fn new() -> Self {
		Self(RawBTree::new())
	}

	pub fn len(&self) -> usize {
		self.0.len()
	}

	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	pub fn iter(&self) -> Iter<R, V, T> {
		Iter(self.0.iter())
	}
}

impl<R, V, T> Default for TypedMap<R, V, T> {
	fn default() -> Self {
		Self::new()
	}
}

impl<R: Ord, V, T> TypedMap<R, V, T> {
	pub fn get_entry_untyped(
		&self,
		key: &impl ValueLike<Resource = R>,
	) -> Option<(&TypedValue<R, T>, &V)> {
		self.0.get(Entry::untyped_cmp, key).map(Entry::as_pair)
	}

	pub fn get_untyped(&self, key: &impl ValueLike<Resource = R>) -> Option<&V> {
		Some(self.get_entry_untyped(key)?.1)
	}

	pub fn insert(&mut self, key: TypedValue<R, T>, value: V) -> Option<(TypedValue<R, T>, V)> {
		self.0
			.insert(Entry::untyped_entry_cmp, Entry::new(key, value))
			.map(Entry::into_pair)
	}

	// pub fn pattern_matching<'a>(&'a self, pattern: &'a TypedPattern<R>) -> impl 'a + Iterator<Item = (&TypedValue<R, T>, &V)> {
	// 	self.iter().filter(|(key, _)| pattern.matches(key))
	// }
}

impl<R: Ord, V, T: PartialEq> TypedMap<R, V, T> {
	pub fn get_entry(&self, key: &TypedValue<R, T>) -> Option<(&TypedValue<R, T>, &V)> {
		self.get_entry_untyped(key)
			.filter(|(entry_key, _)| *entry_key == key)
	}

	pub fn get(&self, key: &TypedValue<R, T>) -> Option<&V> {
		Some(self.get_entry(key)?.1)
	}
}

impl<R: fmt::Debug, V: fmt::Debug, T: fmt::Debug> fmt::Debug for TypedMap<R, V, T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_map().entries(self.iter()).finish()
	}
}

impl<R: PartialEq, V: PartialEq, T: PartialEq> PartialEq for TypedMap<R, V, T> {
	fn eq(&self, other: &Self) -> bool {
		self.iter().eq(other.iter())
	}
}

impl<R: Eq, V: Eq, T: Eq> Eq for TypedMap<R, V, T> {}

impl<R: 'static + PartialOrd, V: PartialOrd, T: PartialOrd> PartialOrd for TypedMap<R, V, T> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		self.iter().partial_cmp(other.iter())
	}
}

impl<R: 'static + Ord, V: Ord, T: Ord> Ord for TypedMap<R, V, T> {
	fn cmp(&self, other: &Self) -> Ordering {
		self.iter().cmp(other.iter())
	}
}

impl<R: Hash, V: Hash, T: Hash> Hash for TypedMap<R, V, T> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		for entry in &self.0 {
			entry.hash(state);
		}
	}
}

impl<R: Ord, V, T: Ord> Extend<(TypedValue<R, T>, V)> for TypedMap<R, V, T> {
	fn extend<I: IntoIterator<Item = (TypedValue<R, T>, V)>>(&mut self, iter: I) {
		for (key, value) in iter {
			self.insert(key, value);
		}
	}
}

impl<R: Ord, V, T: Ord> FromIterator<(TypedValue<R, T>, V)> for TypedMap<R, V, T> {
	fn from_iter<I: IntoIterator<Item = (TypedValue<R, T>, V)>>(iter: I) -> Self {
		let mut result = Self::default();
		result.extend(iter);
		result
	}
}

pub struct Iter<'a, R, V, T>(raw_btree::Iter<'a, Entry<R, V, T>>);

impl<'a, R, V, T> Iterator for Iter<'a, R, V, T> {
	type Item = (&'a TypedValue<R, T>, &'a V);

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next().map(Entry::as_pair)
	}
}

impl<'a, R, V, T> IntoIterator for &'a TypedMap<R, V, T> {
	type Item = (&'a TypedValue<R, T>, &'a V);
	type IntoIter = Iter<'a, R, V, T>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

pub struct IntoIter<R, V, T>(raw_btree::IntoIter<Entry<R, V, T>>);

impl<R, V, T> Iterator for IntoIter<R, V, T> {
	type Item = (TypedValue<R, T>, V);

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next().map(Entry::into_pair)
	}
}

impl<R, V, T> IntoIterator for TypedMap<R, V, T> {
	type Item = (TypedValue<R, T>, V);
	type IntoIter = IntoIter<R, V, T>;

	fn into_iter(self) -> Self::IntoIter {
		IntoIter(self.0.into_iter())
	}
}

#[derive(Clone, Hash)]
struct Entry<R, V, T> {
	key: TypedValue<R, T>,
	value: V,
}

impl<R, V, T> Entry<R, V, T> {
	fn new(key: TypedValue<R, T>, value: V) -> Self {
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

	fn as_pair(&self) -> (&TypedValue<R, T>, &V) {
		(&self.key, &self.value)
	}

	fn into_pair(self) -> (TypedValue<R, T>, V) {
		(self.key, self.value)
	}
}
