use std::{
	cmp::Ordering,
	collections::{btree_map::Entry, BTreeMap},
};

use locspan::{Meta, StrippedPartialEq};

use crate::{metadata::Merge, Id, Multiple};

use super::{PropertyValue, PropertyValueRef};

/// Property values.
#[derive(Clone, Debug)]
pub struct PropertyValues<T, M> {
	/// Base property values (mapped to their metadata).
	base: BTreeMap<T, M>,

	/// Sub-properties values.
	sub_properties: BTreeMap<Id, Self>,
}

impl<T, M> Default for PropertyValues<T, M> {
	fn default() -> Self {
		Self {
			base: BTreeMap::new(),
			sub_properties: BTreeMap::new(),
		}
	}
}

impl<T, M> PropertyValues<T, M> {
	pub fn first(&self) -> Option<PropertyValueRef<T, M>> {
		self.iter().next()
	}

	pub fn len(&self) -> usize {
		let mut len = self.base.len();

		for s in self.sub_properties.values() {
			len += s.len()
		}

		len
	}

	pub fn is_empty(&self) -> bool {
		self.base.is_empty() && self.sub_properties.is_empty()
	}

	pub fn iter(&self) -> Iter<T, M> {
		Iter {
			len: self.len(),
			base: self.base.iter(),
			sub_properties: self.sub_properties.iter(),
			sub_property: None,
			back_sub_property: None,
		}
	}

	pub fn clear(&mut self) {
		self.base.clear();
		self.sub_properties.clear()
	}

	pub fn keep_max_with(&mut self, cmp: impl Fn(&T, &T) -> Option<Ordering>)
	where
		T: Ord,
	{
		let map = std::mem::take(&mut self.base);
		for (item, m) in map {
			let mut insert = true;
			self.base.retain(|other, _| match cmp(other, &item) {
				None => true,
				Some(Ordering::Greater) | Some(Ordering::Equal) => {
					insert = false;
					true
				}
				Some(Ordering::Less) => false,
			});

			if insert {
				self.base.insert(item, m);
			}
		}
	}

	pub fn keep_min_with(&mut self, cmp: impl Fn(&T, &T) -> Option<Ordering>)
	where
		T: Ord,
	{
		let map = std::mem::take(&mut self.base);
		for (item, m) in map {
			let mut insert = true;
			self.base.retain(|other, _| match cmp(other, &item) {
				None => true,
				Some(Ordering::Less) | Some(Ordering::Equal) => {
					insert = false;
					true
				}
				Some(Ordering::Greater) => false,
			});

			if insert {
				self.base.insert(item, m);
			}
		}
	}

	pub fn clone_as_multiple(&self) -> Multiple<T, M>
	where
		T: Clone + Ord,
		M: Clone + Merge,
	{
		self.iter().map(|v| v.value.cloned()).collect()
	}
}

impl<T: Ord, M> PropertyValues<T, M> {
	pub fn contains(&self, item: &T) -> bool {
		self.base.contains_key(item) || self.sub_properties.values().any(|s| s.contains(item))
	}

	pub fn get_metadata(&self, item: &T) -> Option<&M> {
		self.base.get(item).or_else(|| {
			self.sub_properties
				.values()
				.find_map(|s| s.get_metadata(item))
		})
	}

	pub fn insert_base(&mut self, Meta(value, meta): Meta<T, M>)
	where
		M: Merge,
	{
		match self.base.entry(value) {
			Entry::Occupied(mut entry) => entry.get_mut().merge_with(meta),
			Entry::Vacant(entry) => {
				entry.insert(meta);
			}
		}
	}

	/// Inserts a value `v` for a sub property `id`.
	///
	/// The `prop_cmp` function compares properties according to the partial
	/// order defined by the `rdfs:subPropertyOf` relation.
	pub fn insert(
		&mut self,
		id: Option<Id>,
		prop_cmp: impl Fn(Id, Id) -> Option<Ordering>,
		v: Meta<T, M>,
	) where
		M: Merge,
	{
		match id {
			None => self.insert_base(v),
			Some(id) => match self.sub_properties.get_mut(&id) {
				Some(s) => s.insert_base(v),
				None => {
					let mut sub_properties = Vec::new();
					for (other_id, s) in &mut self.sub_properties {
						match prop_cmp(id, *other_id) {
							Some(Ordering::Less) => {
								s.insert(Some(id), prop_cmp, v);
								return;
							}
							Some(Ordering::Equal) => {
								panic!("same property with different IDs")
							}
							Some(Ordering::Greater) => sub_properties.push(*other_id),
							None => (),
						}
					}

					let mut s = Self::default();
					s.insert_base(v);

					for other_id in sub_properties {
						s.sub_properties
							.insert(other_id, self.sub_properties.remove(&other_id).unwrap());
					}

					self.sub_properties.insert(id, s);
				}
			},
		}
	}

	pub fn insert_base_unique(&mut self, Meta(value, meta): Meta<T, M>) -> Option<M> {
		self.base.insert(value, meta)
	}

	pub fn replace_base(&mut self, Meta(value, meta): Meta<T, M>) {
		self.clear();
		self.base.insert(value, meta);
	}

	pub fn remove_base(&mut self, t: &T) -> Option<M> {
		self.base.remove(t)
	}

	pub fn intersected_with_base<I: IntoIterator<Item = Meta<T, M>>>(mut self, iter: I) -> Self
	where
		M: Merge,
	{
		iter.into_iter()
			.filter_map(|Meta(k, m1)| self.base.remove(&k).map(|m2| Meta(k, m1.merged_with(m2))))
			.collect()
	}

	pub fn extended_with_base<I: IntoIterator<Item = Meta<T, M>>>(mut self, iter: I) -> Self
	where
		M: Merge,
	{
		self.extend(iter);
		self
	}
}

impl<T: Ord, M, N> PartialEq<PropertyValues<T, N>> for PropertyValues<T, M> {
	fn eq(&self, other: &PropertyValues<T, N>) -> bool {
		StrippedPartialEq::stripped_eq(self, other)
	}
}

impl<T: Ord, M, N> StrippedPartialEq<PropertyValues<T, N>> for PropertyValues<T, M> {
	fn stripped_eq(&self, other: &PropertyValues<T, N>) -> bool {
		if !self.base.keys().any(|k| other.base.contains_key(k))
			|| self.sub_properties.len() != other.sub_properties.len()
		{
			return false;
		}

		for (id, s1) in &self.sub_properties {
			match other.sub_properties.get(id) {
				Some(s2) => {
					if s1 != s2 {
						return false;
					}
				}
				None => return false,
			}
		}

		true
	}
}

impl<T: Ord, M> From<Meta<T, M>> for PropertyValues<T, M> {
	fn from(v: Meta<T, M>) -> Self {
		let mut result = Self::default();
		result.insert_base_unique(v);
		result
	}
}

impl<T: Ord, M: Merge> Extend<Meta<T, M>> for PropertyValues<T, M> {
	fn extend<I: IntoIterator<Item = Meta<T, M>>>(&mut self, iter: I) {
		for t in iter {
			self.insert_base(t)
		}
	}
}

impl<T: Ord, M: Merge> FromIterator<Meta<T, M>> for PropertyValues<T, M> {
	fn from_iter<I: IntoIterator<Item = Meta<T, M>>>(iter: I) -> Self {
		let mut result = Self::default();

		result.extend(iter);

		result
	}
}

impl<T, M> IntoIterator for PropertyValues<T, M> {
	type Item = PropertyValue<T, M>;
	type IntoIter = IntoIter<T, M>;

	fn into_iter(self) -> Self::IntoIter {
		IntoIter {
			len: self.len(),
			base: self.base.into_iter(),
			sub_properties: self.sub_properties.into_iter(),
			sub_property: None,
			back_sub_property: None,
		}
	}
}

impl<'a, T, M> IntoIterator for &'a PropertyValues<T, M> {
	type Item = PropertyValueRef<'a, T, M>;
	type IntoIter = Iter<'a, T, M>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

pub struct Iter<'a, T, M> {
	len: usize,
	base: std::collections::btree_map::Iter<'a, T, M>,
	sub_properties: std::collections::btree_map::Iter<'a, Id, PropertyValues<T, M>>,
	sub_property: Option<(Id, Box<Self>)>,
	back_sub_property: Option<(Id, Box<Self>)>,
}

impl<'a, T, M> Iterator for Iter<'a, T, M> {
	type Item = PropertyValueRef<'a, T, M>;

	fn size_hint(&self) -> (usize, Option<usize>) {
		(self.len, Some(self.len))
	}

	fn next(&mut self) -> Option<Self::Item> {
		self.base
			.next()
			.map(|(v, m)| {
				self.len -= 1;
				PropertyValueRef {
					sub_property: None,
					value: Meta(v, m),
				}
			})
			.or_else(|| loop {
				match &mut self.sub_property {
					Some((id, s)) => match s.next() {
						Some(v) => {
							self.len -= 1;
							break Some(v.for_sub_property(*id));
						}
						None => self.sub_property = None,
					},
					None => {
						self.sub_property = self
							.sub_properties
							.next()
							.map(|(id, s)| (*id, Box::new(s.iter())))
							.or_else(|| self.back_sub_property.take())
					}
				}
			})
	}
}

impl<'a, T, M> ExactSizeIterator for Iter<'a, T, M> {}

impl<'a, T, M> DoubleEndedIterator for Iter<'a, T, M> {
	fn next_back(&mut self) -> Option<Self::Item> {
		loop {
			match &mut self.back_sub_property {
				Some((id, s)) => match s.next_back() {
					Some(v) => {
						self.len -= 1;
						break Some(v.for_sub_property(*id));
					}
					None => self.back_sub_property = None,
				},
				None => {
					self.back_sub_property = self
						.sub_properties
						.next_back()
						.map(|(id, s)| (*id, Box::new(s.iter())))
						.or_else(|| self.sub_property.take())
				}
			}
		}
		.or_else(|| {
			self.base.next().map(|(v, m)| {
				self.len -= 1;
				PropertyValueRef {
					sub_property: None,
					value: Meta(v, m),
				}
			})
		})
	}
}

pub struct IntoIter<T, M> {
	len: usize,
	base: std::collections::btree_map::IntoIter<T, M>,
	sub_properties: std::collections::btree_map::IntoIter<Id, PropertyValues<T, M>>,
	sub_property: Option<(Id, Box<Self>)>,
	back_sub_property: Option<(Id, Box<Self>)>,
}

impl<T, M> Iterator for IntoIter<T, M> {
	type Item = PropertyValue<T, M>;

	fn size_hint(&self) -> (usize, Option<usize>) {
		(self.len, Some(self.len))
	}

	fn next(&mut self) -> Option<Self::Item> {
		self.base
			.next()
			.map(|(v, m)| {
				self.len -= 1;
				PropertyValue {
					sub_property: None,
					value: Meta(v, m),
				}
			})
			.or_else(|| loop {
				match &mut self.sub_property {
					Some((id, s)) => match s.next() {
						Some(v) => {
							self.len -= 1;
							break Some(v.for_sub_property(*id));
						}
						None => self.sub_property = None,
					},
					None => {
						self.sub_property = self
							.sub_properties
							.next()
							.map(|(id, s)| (id, Box::new(s.into_iter())))
							.or_else(|| self.back_sub_property.take())
					}
				}
			})
	}
}

impl<T, M> ExactSizeIterator for IntoIter<T, M> {}

impl<T, M> DoubleEndedIterator for IntoIter<T, M> {
	fn next_back(&mut self) -> Option<Self::Item> {
		loop {
			match &mut self.back_sub_property {
				Some((id, s)) => match s.next_back() {
					Some(v) => {
						self.len -= 1;
						break Some(v.for_sub_property(*id));
					}
					None => self.back_sub_property = None,
				},
				None => {
					self.back_sub_property = self
						.sub_properties
						.next_back()
						.map(|(id, s)| (id, Box::new(s.into_iter())))
						.or_else(|| self.sub_property.take())
				}
			}
		}
		.or_else(|| {
			self.base.next().map(|(v, m)| {
				self.len -= 1;
				PropertyValue {
					sub_property: None,
					value: Meta(v, m),
				}
			})
		})
	}
}
