use std::collections::BTreeMap;

use educe::Educe;

use crate::Value;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Educe)]
#[educe(Default)]
pub struct Attributes<R> {
	map: BTreeMap<Attribute<R>, Vec<Value<R>>>,
}

impl<R> Attributes<R> {
	pub fn new() -> Self {
		Self {
			map: BTreeMap::new(),
		}
	}

	pub fn is_empty(&self) -> bool {
		self.map.is_empty()
	}
}

impl<R: Ord> Attributes<R> {
	pub fn insert(&mut self, attr: Attribute<R>, value: Value<R>) {
		self.map.entry(attr).or_default().push(value)
	}

	pub fn extend_attribute(
		&mut self,
		attr: Attribute<R>,
		values: impl IntoIterator<Item = Value<R>>,
	) {
		self.map.entry(attr).or_default().extend(values)
	}
}

pub type IntoIter<R> = std::collections::btree_map::IntoIter<Attribute<R>, Vec<Value<R>>>;

impl<R> IntoIterator for Attributes<R> {
	type IntoIter = IntoIter<R>;
	type Item = (Attribute<R>, Vec<Value<R>>);

	fn into_iter(self) -> Self::IntoIter {
		self.map.into_iter()
	}
}

impl<R, V> Extend<(Attribute<R>, V)> for Attributes<R>
where
	R: Ord,
	V: IntoIterator<Item = Value<R>>,
{
	fn extend<T: IntoIterator<Item = (Attribute<R>, V)>>(&mut self, iter: T) {
		for (attr, values) in iter {
			self.extend_attribute(attr, values);
		}
	}
}

impl<R, V> FromIterator<(Attribute<R>, V)> for Attributes<R>
where
	R: Ord,
	V: IntoIterator<Item = Value<R>>,
{
	fn from_iter<T: IntoIterator<Item = (Attribute<R>, V)>>(iter: T) -> Self {
		let mut result = Self::new();
		result.extend(iter);
		result
	}
}

impl<R> Extend<(Attribute<R>, Value<R>)> for Attributes<R>
where
	R: Ord,
{
	fn extend<T: IntoIterator<Item = (Attribute<R>, Value<R>)>>(&mut self, iter: T) {
		for (attr, value) in iter {
			self.insert(attr, value);
		}
	}
}

impl<R> FromIterator<(Attribute<R>, Value<R>)> for Attributes<R>
where
	R: Ord,
{
	fn from_iter<T: IntoIterator<Item = (Attribute<R>, Value<R>)>>(iter: T) -> Self {
		let mut result = Self::new();
		result.extend(iter);
		result
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Attribute<R> {
	Documentation,
	Custom(R),
}
