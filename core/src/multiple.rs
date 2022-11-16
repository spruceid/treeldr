use locspan::{Meta, StrippedPartialEq};
use std::collections::{btree_map::Entry, BTreeMap};
use crate::metadata::Merge;

/// Multiple values.
#[derive(Clone, Debug)]
pub struct Multiple<T, M>(BTreeMap<T, M>);

impl<T, M> Default for Multiple<T, M> {
	fn default() -> Self {
		Self(BTreeMap::new())
	}
}

impl<T, M> Multiple<T, M> {
	pub fn first(&self) -> Option<Meta<&T, &M>> {
		self.iter().next()
	}

	pub fn len(&self) -> usize {
		self.0.len()
	}
	
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	pub fn iter(&self) -> Iter<T, M> {
		Iter(self.0.iter())
	}

	pub fn clear(&mut self) {
		self.0.clear()
	}
}

impl<T: Ord, M> Multiple<T, M> {
	pub fn contains(&self, item: &T) -> bool {
		self.0.contains_key(item)
	}
	
	pub fn insert(&mut self, Meta(value, meta): Meta<T, M>)
	where
		M: Merge,
	{
		match self.0.entry(value) {
			Entry::Occupied(mut entry) => entry.get_mut().merge_with(meta),
			Entry::Vacant(entry) => {
				entry.insert(meta);
			}
		}
	}

	pub fn replace(&mut self, Meta(value, meta): Meta<T, M>) {
		self.0.clear();
		self.0.insert(value, meta);
	}

	pub fn intersected_with<I: IntoIterator<Item = Meta<T, M>>>(mut self, iter: I) -> Self
	where
		M: Merge,
	{
		iter.into_iter().filter_map(|Meta(k, m1)| {
			self.0.remove(&k).map(|m2| Meta(k, m1.merged_with(m2)))
		}).collect()
	}

	pub fn extended_with<I: IntoIterator<Item = Meta<T, M>>>(mut self, iter: I) -> Self
	where
		M: Merge,
	{
		self.extend(iter);
		self
	}
}

impl<T: Ord, M, N> PartialEq<Multiple<T, N>> for Multiple<T, M> {
	fn eq(&self, other: &Multiple<T, N>) -> bool {
		self.0.keys().any(|k| other.0.contains_key(k))
	}
}

impl<T: Ord, M, N> StrippedPartialEq<Multiple<T, N>> for Multiple<T, M> {
	fn stripped_eq(&self, other: &Multiple<T, N>) -> bool {
		self.0.keys().any(|k| other.0.contains_key(k))
	}
}

impl<T: Ord, M> From<Meta<T, M>> for Multiple<T, M> {
	fn from(Meta(t, m): Meta<T, M>) -> Self {
		Self(Some((t, m)).into_iter().collect())
	}
}

impl<T: Ord, M: Merge> Extend<Meta<T, M>> for Multiple<T, M> {
	fn extend<I: IntoIterator<Item = Meta<T, M>>>(&mut self, iter: I) {
		for t in iter {
			self.insert(t)
		}
	}
}

impl<T: Ord, M: Merge> FromIterator<Meta<T, M>> for Multiple<T, M> {
	fn from_iter<I: IntoIterator<Item = Meta<T, M>>>(iter: I) -> Self {
		let mut result = Self::default();
		
		result.extend(iter);

		result
	}
}

impl<T, M> IntoIterator for Multiple<T, M> {
	type Item = Meta<T, M>;
	type IntoIter = IntoIter<T, M>;

	fn into_iter(self) -> Self::IntoIter {
		IntoIter(self.0.into_iter())
	}
}

impl<'a, T, M> IntoIterator for &'a Multiple<T, M> {
	type Item = Meta<&'a T, &'a M>;
	type IntoIter = Iter<'a, T, M>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

pub struct Iter<'a, T, M>(std::collections::btree_map::Iter<'a, T, M>);

impl<'a, T, M> Iterator for Iter<'a, T, M> {
	type Item = Meta<&'a T, &'a M>;

	fn size_hint(&self) -> (usize, Option<usize>) {
		self.0.size_hint()
	}

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next().map(|(t, m)| Meta(t, m))
	}
}

impl<'a, T, M> ExactSizeIterator for Iter<'a, T, M> {}

impl<'a, T, M> DoubleEndedIterator for Iter<'a, T, M> {
	fn next_back(&mut self) -> Option<Self::Item> {
		self.0.next_back().map(|(t, m)| Meta(t, m))
	}
}

pub struct IntoIter<T, M>(std::collections::btree_map::IntoIter<T, M>);

impl<T, M> Iterator for IntoIter<T, M> {
	type Item = Meta<T, M>;

	fn size_hint(&self) -> (usize, Option<usize>) {
		self.0.size_hint()
	}

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next().map(|(t, m)| Meta(t, m))
	}
}

impl<T, M> ExactSizeIterator for IntoIter<T, M> {}

impl<T, M> DoubleEndedIterator for IntoIter<T, M> {
	fn next_back(&mut self) -> Option<Self::Item> {
		self.0.next_back().map(|(t, m)| Meta(t, m))
	}
}
