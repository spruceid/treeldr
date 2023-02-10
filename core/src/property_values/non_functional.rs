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

pub type TryMapError<M, U, E> = (Meta<E, M>, PropertyValues<U, M>);
pub type TryMappedError<'a, M, U, N, E> = (Meta<E, &'a M>, PropertyValues<U, N>);

impl<T, M> PropertyValues<T, M> {
	pub fn from_base(base: BTreeMap<T, M>) -> Self {
		Self {
			base,
			sub_properties: BTreeMap::default(),
		}
	}

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

	pub fn map<U>(self, mut f: impl FnMut(T) -> U) -> PropertyValues<U, M>
	where
		U: Ord,
	{
		self.map_with(&mut f)
	}

	fn map_with<U>(self, f: &mut impl FnMut(T) -> U) -> PropertyValues<U, M>
	where
		U: Ord,
	{
		let base = self.base.into_iter().map(|(k, v)| (f(k), v)).collect();
		let sub_properties = self
			.sub_properties
			.into_iter()
			.map(|(id, m)| (id, m.map_with(f)))
			.collect();

		PropertyValues {
			base,
			sub_properties,
		}
	}

	pub fn mapped<U, N>(
		&self,
		mut f: impl FnMut(Meta<&T, &M>) -> Meta<U, N>,
	) -> PropertyValues<U, N>
	where
		U: Ord,
	{
		self.mapped_with(&mut f)
	}

	fn mapped_with<U, N>(
		&self,
		f: &mut impl FnMut(Meta<&T, &M>) -> Meta<U, N>,
	) -> PropertyValues<U, N>
	where
		U: Ord,
	{
		let base = self
			.base
			.iter()
			.map(|(t, m)| {
				let Meta(u, n) = f(Meta(t, m));
				(u, n)
			})
			.collect();
		let sub_properties = self
			.sub_properties
			.iter()
			.map(|(id, m)| (*id, m.mapped_with(f)))
			.collect();

		PropertyValues {
			base,
			sub_properties,
		}
	}

	pub fn map_properties<U>(
		self,
		mut g: impl FnMut(Id) -> Id,
		mut f: impl FnMut(T) -> U,
	) -> PropertyValues<U, M>
	where
		U: Ord,
	{
		self.map_properties_with(&mut g, &mut f)
	}

	fn map_properties_with<U>(
		self,
		g: &mut impl FnMut(Id) -> Id,
		f: &mut impl FnMut(T) -> U,
	) -> PropertyValues<U, M>
	where
		U: Ord,
	{
		let base = self.base.into_iter().map(|(k, v)| (f(k), v)).collect();
		let sub_properties = self
			.sub_properties
			.into_iter()
			.map(|(id, m)| (g(id), m.map_with(f)))
			.collect();

		PropertyValues {
			base,
			sub_properties,
		}
	}

	pub fn try_map<U, E>(
		self,
		mut f: impl FnMut(Option<Id>, T) -> Result<U, E>,
	) -> Result<PropertyValues<U, M>, TryMapError<M, U, E>>
	where
		U: Ord,
	{
		self.try_map_with(None, &mut f)
	}

	fn try_map_with<U, E>(
		self,
		id: Option<Id>,
		f: &mut impl FnMut(Option<Id>, T) -> Result<U, E>,
	) -> Result<PropertyValues<U, M>, TryMapError<M, U, E>>
	where
		U: Ord,
	{
		let mut base = BTreeMap::new();
		for (t, m) in self.base {
			match f(id, t) {
				Ok(u) => {
					base.insert(u, m);
				}
				Err(e) => {
					return Err((
						Meta(e, m),
						PropertyValues {
							base,
							sub_properties: BTreeMap::new(),
						},
					));
				}
			}
		}

		let mut sub_properties = BTreeMap::new();
		for (id, t) in self.sub_properties {
			match t.try_map_with(Some(id), f) {
				Ok(u) => {
					sub_properties.insert(id, u);
				}
				Err((e, u)) => {
					sub_properties.insert(id, u);
					return Err((
						e,
						PropertyValues {
							base,
							sub_properties,
						},
					));
				}
			}
		}

		Ok(PropertyValues {
			base,
			sub_properties,
		})
	}

	pub fn try_mapped<'a, U, N, E>(
		&'a self,
		mut f: impl FnMut(Option<Id>, Meta<&'a T, &'a M>) -> Result<Meta<U, N>, E>,
	) -> Result<PropertyValues<U, N>, TryMappedError<'a, M, U, N, E>>
	where
		U: Ord,
	{
		self.try_mapped_with(None, &mut f)
	}

	fn try_mapped_with<'a, U, N, E>(
		&'a self,
		id: Option<Id>,
		f: &mut impl FnMut(Option<Id>, Meta<&'a T, &'a M>) -> Result<Meta<U, N>, E>,
	) -> Result<PropertyValues<U, N>, TryMappedError<'a, M, U, N, E>>
	where
		U: Ord,
	{
		let mut base = BTreeMap::new();
		for (t, m) in &self.base {
			match f(id, Meta(t, m)) {
				Ok(Meta(u, n)) => {
					base.insert(u, n);
				}
				Err(e) => {
					return Err((
						Meta(e, m),
						PropertyValues {
							base,
							sub_properties: BTreeMap::new(),
						},
					));
				}
			}
		}

		let mut sub_properties = BTreeMap::new();
		for (id, t) in &self.sub_properties {
			match t.try_mapped_with(Some(*id), f) {
				Ok(u) => {
					sub_properties.insert(*id, u);
				}
				Err((e, u)) => {
					sub_properties.insert(*id, u);
					return Err((
						e,
						PropertyValues {
							base,
							sub_properties,
						},
					));
				}
			}
		}

		Ok(PropertyValues {
			base,
			sub_properties,
		})
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

	pub fn replace_with_base(&mut self, Meta(value, meta): Meta<T, M>) {
		self.clear();
		self.base.insert(value, meta);
	}

	pub fn remove(&mut self, t: &T) -> Option<M> {
		let mut result = self.base.remove(t);

		let mut reinsert = Vec::new();
		self.sub_properties.retain(|_, s| {
			if let Some(r) = s.remove(t) {
				result = Some(r)
			}

			if s.base.is_empty() {
				reinsert.extend(std::mem::take(&mut s.sub_properties));
				false
			} else {
				true
			}
		});

		for (id, s) in reinsert {
			self.sub_properties.insert(id, s);
		}

		result
	}

	pub fn retain(&mut self, mut f: impl FnMut(&T) -> bool) {
		self.retain_with(&mut f)
	}

	fn retain_with(&mut self, f: &mut impl FnMut(&T) -> bool) {
		self.base.retain(|t, _| f(t));

		let mut reinsert = Vec::new();
		self.sub_properties.retain(|_, s| {
			s.retain_with(f);
			if s.base.is_empty() {
				reinsert.extend(std::mem::take(&mut s.sub_properties));
				false
			} else {
				true
			}
		});

		for (id, s) in reinsert {
			self.sub_properties.insert(id, s);
		}
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
						let sub_property = self
							.sub_properties
							.next()
							.map(|(id, s)| (*id, Box::new(s.iter())))
							.or_else(|| self.back_sub_property.take());

						match sub_property {
							Some(s) => self.sub_property = Some(s),
							None => break None,
						}
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
					let back_sub_property = self
						.sub_properties
						.next_back()
						.map(|(id, s)| (*id, Box::new(s.iter())))
						.or_else(|| self.sub_property.take());

					match back_sub_property {
						Some(s) => self.back_sub_property = Some(s),
						None => break None,
					}
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
						let sub_property = self
							.sub_properties
							.next()
							.map(|(id, s)| (id, Box::new(s.into_iter())))
							.or_else(|| self.back_sub_property.take());

						match sub_property {
							Some(s) => self.sub_property = Some(s),
							None => break None,
						}
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
					let back_sub_property = self
						.sub_properties
						.next_back()
						.map(|(id, s)| (id, Box::new(s.into_iter())))
						.or_else(|| self.sub_property.take());

					match back_sub_property {
						Some(s) => self.back_sub_property = Some(s),
						None => break None,
					}
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
