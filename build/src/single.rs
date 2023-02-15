use locspan::{ErrAt, Meta, StrippedEq, StrippedOrd, StrippedPartialEq, StrippedPartialOrd};
use std::collections::{btree_map::Entry, BTreeMap};
use treeldr::{metadata::Merge, Id, MetaOption, PropertyValue};

use crate::{
	error::{self, NodeBindingFunctionalConflict},
	Context, Error, FunctionalPropertyValue, ListRef, Property,
};

#[derive(Debug, Clone)]
pub struct Conflict<T, M>(pub Meta<T, M>, pub Meta<T, M>);

impl<T, M> Conflict<T, M> {
	pub fn at_functional_node_property(self, id: Id, property: impl Into<Property>) -> Error<M>
	where
		(T, PropertyValue<T, M>):
			Into<crate::error::node_binding_functional_conflict::ConflictValues<M>>,
	{
		let Meta(a, meta) = self.0;

		Meta(
			NodeBindingFunctionalConflict {
				id,
				property: property.into(),
				values: (a, PropertyValue::new(None, self.1)).into(),
			}
			.into(),
			meta,
		)
	}
}

/// Single value with multiple representatives.
///
/// It is expected that all the representatives collapse to the same value
/// after the model simplification phase.
///
/// # Equality
///
/// Because all the representatives are logically equals, it is sufficient for
/// two `Single` values to share one representative to be considered equals.
///
/// There is no total equality.
#[derive(Clone, Debug)]
pub struct Single<T, M>(BTreeMap<T, M>);

impl<T, M> Default for Single<T, M> {
	fn default() -> Self {
		Self(BTreeMap::new())
	}
}

impl<T, M> Single<T, M> {
	pub fn first(&self) -> Option<Meta<&T, &M>> {
		self.iter().next()
	}

	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	pub fn len(&self) -> usize {
		self.0.len()
	}

	pub fn clear(&mut self) {
		self.0.clear()
	}

	pub fn iter(&self) -> Iter<T, M> {
		Iter(self.0.iter())
	}

	pub fn into_functional_property_value(self) -> FunctionalPropertyValue<T, M> {
		FunctionalPropertyValue::from_base(self.0)
	}
}

impl<T: Ord, M> Single<T, M> {
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

	pub fn extended_with<I: IntoIterator<Item = Meta<T, M>>>(mut self, iter: I) -> Self
	where
		M: Merge,
	{
		self.extend(iter);
		self
	}
}

impl<T, M> Single<T, M> {
	pub fn try_unwrap(self) -> Result<MetaOption<T, M>, Conflict<T, M>> {
		let mut it = self.into_iter();
		match it.next() {
			Some(a) => match it.next() {
				Some(b) => Err(Conflict(a, b)),
				None => Ok(Some(a).into()),
			},
			None => Ok(None.into()),
		}
	}

	pub fn try_unwraped(&self) -> Result<Option<Meta<&T, &M>>, Conflict<T, M>>
	where
		T: Clone,
		M: Clone,
	{
		let mut it = self.iter();
		match it.next() {
			Some(a) => match it.next() {
				Some(b) => Err(Conflict(a.cloned(), b.cloned())),
				None => Ok(Some(a)),
			},
			None => Ok(None),
		}
	}

	pub fn as_required_at_node_binding(
		&self,
		id: Id,
		prop: impl Copy + Into<Property>,
		meta: &M,
	) -> Result<Meta<&T, &M>, Error<M>>
	where
		(T, PropertyValue<T, M>):
			Into<crate::error::node_binding_functional_conflict::ConflictValues<M>>,
		T: Clone,
		M: Clone,
	{
		self.try_unwraped()
			.map_err(|c| c.at_functional_node_property(id, prop))?
			.ok_or_else(|| error::NodeBindingMissing::new(id, prop).into())
			.err_at(|| meta.clone())
	}

	pub fn into_required_at_node_binding(
		self,
		id: Id,
		prop: impl Copy + Into<Property>,
		meta: &M,
	) -> Result<Meta<T, M>, Error<M>>
	where
		(T, PropertyValue<T, M>):
			Into<crate::error::node_binding_functional_conflict::ConflictValues<M>>,
		M: Clone,
	{
		self.try_unwrap()
			.map_err(|c| c.at_functional_node_property(id, prop))?
			.ok_or_else(|| error::NodeBindingMissing::new(id, prop).into())
			.err_at(|| meta.clone())
	}
}

impl<M: Clone> Single<Id, M> {
	pub fn into_type_at_node_binding(
		self,
		context: &Context<M>,
		id: Id,
		prop: impl Copy + Into<Property>,
	) -> Result<MetaOption<treeldr::TId<treeldr::Type>, M>, Error<M>> {
		self.try_unwrap()
			.map_err(|c| c.at_functional_node_property(id, prop))?
			.try_map_borrow_metadata(|p, meta| {
				context
					.require_type_id(p)
					.map_err(|e| e.at_node_property(id, prop, meta.clone()))
			})
	}

	pub fn into_datatype_at_node_binding(
		self,
		context: &Context<M>,
		id: Id,
		prop: impl Copy + Into<Property>,
	) -> Result<MetaOption<treeldr::TId<treeldr::ty::DataType<M>>, M>, Error<M>> {
		self.try_unwrap()
			.map_err(|c| c.at_functional_node_property(id, prop))?
			.try_map_borrow_metadata(|p, meta| {
				context
					.require_datatype_id(p)
					.map_err(|e| e.at_node_property(id, prop, meta.clone()))
			})
	}

	pub fn into_required_type_at_node_binding(
		self,
		context: &Context<M>,
		id: Id,
		prop: impl Copy + Into<Property>,
		meta: &M,
	) -> Result<Meta<treeldr::TId<treeldr::Type>, M>, Error<M>> {
		self.into_type_at_node_binding(context, id, prop)?
			.ok_or_else(|| error::NodeBindingMissing::new(id, prop).into())
			.err_at(|| meta.clone())
	}

	pub fn into_required_datatype_at_node_binding(
		self,
		context: &Context<M>,
		id: Id,
		prop: impl Copy + Into<Property>,
		meta: &M,
	) -> Result<Meta<treeldr::TId<treeldr::ty::DataType<M>>, M>, Error<M>> {
		self.into_datatype_at_node_binding(context, id, prop)?
			.ok_or_else(|| error::NodeBindingMissing::new(id, prop).into())
			.err_at(|| meta.clone())
	}

	pub fn into_property_at_node_binding(
		self,
		context: &Context<M>,
		id: Id,
		prop: impl Copy + Into<Property>,
	) -> Result<MetaOption<treeldr::TId<treeldr::Property>, M>, Error<M>> {
		self.try_unwrap()
			.map_err(|c| c.at_functional_node_property(id, prop))?
			.try_map_borrow_metadata(|p, meta| {
				context
					.require_property_id(p)
					.map_err(|e| e.at_node_property(id, prop, meta.clone()))
			})
	}

	pub fn into_required_property_at_node_binding(
		self,
		context: &Context<M>,
		id: Id,
		prop: impl Copy + Into<Property>,
		meta: &M,
	) -> Result<Meta<treeldr::TId<treeldr::Property>, M>, Error<M>> {
		self.into_property_at_node_binding(context, id, prop)?
			.ok_or_else(|| error::NodeBindingMissing::new(id, prop).into())
			.err_at(|| meta.clone())
	}

	pub fn into_layout_at_node_binding(
		self,
		context: &Context<M>,
		id: Id,
		prop: impl Copy + Into<Property>,
	) -> Result<MetaOption<treeldr::TId<treeldr::Layout>, M>, Error<M>> {
		self.try_unwrap()
			.map_err(|c| c.at_functional_node_property(id, prop))?
			.try_map_borrow_metadata(|p, meta| {
				context
					.require_layout_id(p)
					.map_err(|e| e.at_node_property(id, prop, meta.clone()))
			})
	}

	pub fn into_required_layout_at_node_binding(
		self,
		context: &Context<M>,
		id: Id,
		prop: impl Copy + Into<Property>,
		meta: &M,
	) -> Result<Meta<treeldr::TId<treeldr::Layout>, M>, Error<M>> {
		self.into_layout_at_node_binding(context, id, prop)?
			.ok_or_else(|| error::NodeBindingMissing::new(id, prop).into())
			.err_at(|| meta.clone())
	}

	pub fn into_list_at_node_binding(
		self,
		context: &Context<M>,
		id: Id,
		prop: impl Copy + Into<Property>,
	) -> Result<MetaOption<ListRef<M>, M>, Error<M>> {
		self.try_unwrap()
			.map_err(|c| c.at_functional_node_property(id, prop))?
			.try_map_borrow_metadata(|p, meta| {
				context
					.require_list(p)
					.map_err(|e| e.at_node_property(id, prop, meta.clone()))
			})
	}

	pub fn into_required_list_at_node_binding<'l>(
		self,
		context: &'l Context<M>,
		id: Id,
		prop: impl Copy + Into<Property>,
		meta: &M,
	) -> Result<Meta<ListRef<'l, M>, M>, Error<M>> {
		self.into_list_at_node_binding(context, id, prop)?
			.ok_or_else(|| error::NodeBindingMissing::new(id, prop).into())
			.err_at(|| meta.clone())
	}
}

impl<T: Ord, M, N> PartialEq<Single<T, N>> for Single<T, M> {
	fn eq(&self, other: &Single<T, N>) -> bool {
		self.0.keys().any(|k| other.0.contains_key(k))
	}
}

impl<T: Ord, M, N> StrippedPartialEq<Single<T, N>> for Single<T, M> {
	fn stripped_eq(&self, other: &Single<T, N>) -> bool {
		self.0.keys().any(|k| other.0.contains_key(k))
	}
}

impl<T: Ord, M> StrippedEq for Single<T, M> {}

impl<T: Ord, M, N> StrippedPartialOrd<Single<T, N>> for Single<T, M> {
	fn stripped_partial_cmp(&self, other: &Single<T, N>) -> Option<std::cmp::Ordering> {
		Some(
			self.first()
				.map(Meta::into_value)
				.cmp(&other.first().map(Meta::into_value)),
		)
	}
}

impl<T: Ord, M> StrippedOrd for Single<T, M> {
	fn stripped_cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.first()
			.map(Meta::into_value)
			.cmp(&other.first().map(Meta::into_value))
	}
}

impl<T: Ord, M> From<Meta<T, M>> for Single<T, M> {
	fn from(Meta(t, m): Meta<T, M>) -> Self {
		Self(Some((t, m)).into_iter().collect())
	}
}

impl<T: Ord, M: Merge> Extend<Meta<T, M>> for Single<T, M> {
	fn extend<I: IntoIterator<Item = Meta<T, M>>>(&mut self, iter: I) {
		for t in iter {
			self.insert(t)
		}
	}
}

impl<T: Ord, M: Merge> FromIterator<Meta<T, M>> for Single<T, M> {
	fn from_iter<I: IntoIterator<Item = Meta<T, M>>>(iter: I) -> Self {
		let mut result = Self::default();
		result.extend(iter);
		result
	}
}

impl<T, M> IntoIterator for Single<T, M> {
	type Item = Meta<T, M>;
	type IntoIter = IntoIter<T, M>;

	fn into_iter(self) -> Self::IntoIter {
		IntoIter(self.0.into_iter())
	}
}

impl<'a, T, M> IntoIterator for &'a Single<T, M> {
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
