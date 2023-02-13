use std::{cmp::Ordering, collections::BTreeMap};

use locspan::{ErrAt, Meta, StrippedEq, StrippedPartialEq};
use treeldr::{metadata::Merge, prop::UnknownProperty, Id, TId};

use crate::{
	error::{self, NodeBindingFunctionalConflict},
	Context, Error, ListRef, Property, PropertyValue, PropertyValueRef, PropertyValues, Single,
};

#[derive(Debug, Clone)]
pub struct Conflict<T, M>(pub PropertyValue<T, M>, pub PropertyValue<T, M>);

impl<T, M> Conflict<T, M> {
	pub fn at_functional_node_property(self, id: Id, property: impl Into<Property>) -> Error<M>
	where
		(T, PropertyValue<T, M>):
			Into<crate::error::node_binding_functional_conflict::ConflictValues<M>>,
	{
		let PropertyValue {
			value: Meta(a, meta),
			sub_property,
		} = self.0;

		Meta(
			NodeBindingFunctionalConflict {
				id: sub_property.map(TId::into_id).unwrap_or(id),
				property: property.into(),
				values: (a, self.1).into(),
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
pub struct FunctionalPropertyValue<T, M>(PropertyValues<T, M>);

impl<T, M> Default for FunctionalPropertyValue<T, M> {
	fn default() -> Self {
		Self(PropertyValues::default())
	}
}

impl<T, M> FunctionalPropertyValue<T, M> {
	pub fn from_base(base: BTreeMap<T, M>) -> Self {
		Self(PropertyValues::from_base(base))
	}

	pub fn first(&self) -> Option<PropertyValueRef<T, M>> {
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
		self.0.iter()
	}

	pub fn clone_into_single(&self) -> Single<T, M>
	where
		T: Ord + Clone,
		M: Clone + Merge,
	{
		Single::from_iter(self.iter().map(|p| p.value.cloned()))
	}
}

impl<T, M> From<PropertyValues<T, M>> for FunctionalPropertyValue<T, M> {
	fn from(value: PropertyValues<T, M>) -> Self {
		Self(value)
	}
}

impl<T: Ord, M> FunctionalPropertyValue<T, M> {
	pub fn insert_base(&mut self, value: Meta<T, M>)
	where
		M: Merge,
	{
		self.0.insert_base(value)
	}

	pub fn insert(
		&mut self,
		id: Option<TId<UnknownProperty>>,
		prop_cmp: impl Fn(TId<UnknownProperty>, TId<UnknownProperty>) -> Option<Ordering>,
		value: Meta<T, M>,
	) where
		M: Merge,
	{
		self.0.insert(id, prop_cmp, value)
	}

	pub fn replace_with_base(&mut self, value: Meta<T, M>) {
		self.0.replace_with_base(value)
	}

	pub fn extended_with_base<I: IntoIterator<Item = Meta<T, M>>>(mut self, iter: I) -> Self
	where
		M: Merge,
	{
		self.extend(iter);
		self
	}

	pub fn extend_with<I: IntoIterator<Item = PropertyValue<T, M>>>(
		&mut self,
		prop_cmp: impl Fn(TId<UnknownProperty>, TId<UnknownProperty>) -> Option<Ordering>,
		iter: I,
	) where
		M: Merge,
	{
		for PropertyValue {
			sub_property,
			value,
		} in iter
		{
			self.insert(sub_property, &prop_cmp, value)
		}
	}
}

impl<T, M> FunctionalPropertyValue<T, M> {
	pub fn map<U>(self, f: impl FnMut(T) -> U) -> FunctionalPropertyValue<U, M>
	where
		U: Ord,
	{
		FunctionalPropertyValue(self.0.map(f))
	}

	pub fn map_properties<U>(
		self,
		g: impl FnMut(TId<UnknownProperty>) -> TId<UnknownProperty>,
		f: impl FnMut(T) -> U,
	) -> FunctionalPropertyValue<U, M>
	where
		U: Ord,
	{
		FunctionalPropertyValue(self.0.map_properties(g, f))
	}

	pub fn try_unwrap(self) -> Result<treeldr::FunctionalPropertyValue<T, M>, Conflict<T, M>>
	where
		T: PartialEq,
	{
		let mut value: Option<T> = None;

		let result = self.0.try_map(|id, a| {
			value = match value.take() {
				Some(b) => {
					if a == b {
						Some(a)
					} else {
						return Err((id, a));
					}
				}
				None => Some(a),
			};

			Ok(())
		});

		match result {
			Ok(metadata) => {
				Ok(treeldr::FunctionalPropertyValue::new(value.map(|v| {
					treeldr::RequiredFunctionalPropertyValue::new(metadata, v)
				})))
			}
			Err((Meta((sub_property, a), meta), metadata)) => {
				let a = PropertyValue::new(sub_property, Meta(a, meta));
				let b = metadata.into_iter().next().unwrap().map(|_| value.unwrap());
				Err(Conflict(a, b))
			}
		}
	}

	pub fn try_unwraped(&self) -> Result<treeldr::FunctionalPropertyValue<&T, &M>, Conflict<T, M>>
	where
		T: PartialEq + Clone,
		M: Clone,
	{
		let mut value: Option<&T> = None;

		let result = self.0.try_mapped(|id, Meta(a, m)| {
			value = match value.take() {
				Some(b) => {
					if a == b {
						Some(a)
					} else {
						return Err((id, a));
					}
				}
				None => Some(a),
			};

			Ok(Meta((), m))
		});

		match result {
			Ok(metadata) => {
				Ok(treeldr::FunctionalPropertyValue::new(value.map(|v| {
					treeldr::RequiredFunctionalPropertyValue::new(metadata, v)
				})))
			}
			Err((Meta((sub_property, a), meta), metadata)) => {
				let a = PropertyValue::new(sub_property, Meta(a.clone(), meta.clone()));
				let b = metadata
					.into_iter()
					.next()
					.unwrap()
					.map(|_| value.unwrap().clone())
					.into_cloned_metadata();
				Err(Conflict(a, b))
			}
		}
	}

	pub fn as_required_at_node_binding(
		&self,
		id: Id,
		prop: impl Copy + Into<Property>,
		meta: &M,
	) -> Result<treeldr::RequiredFunctionalPropertyValue<&T, &M>, Error<M>>
	where
		(T, PropertyValue<T, M>):
			Into<crate::error::node_binding_functional_conflict::ConflictValues<M>>,
		T: PartialEq + Clone,
		M: Clone,
	{
		self.try_unwraped()
			.map_err(|c| c.at_functional_node_property(id, prop))?
			.into_required()
			.ok_or_else(|| error::NodeBindingMissing::new(id, prop).into())
			.err_at(|| meta.clone())
	}

	pub fn into_required_at_node_binding(
		self,
		id: Id,
		prop: impl Copy + Into<Property>,
		meta: &M,
	) -> Result<treeldr::RequiredFunctionalPropertyValue<T, M>, Error<M>>
	where
		(T, PropertyValue<T, M>):
			Into<crate::error::node_binding_functional_conflict::ConflictValues<M>>,
		T: PartialEq,
		M: Clone,
	{
		self.try_unwrap()
			.map_err(|c| c.at_functional_node_property(id, prop))?
			.into_required()
			.ok_or_else(|| error::NodeBindingMissing::new(id, prop).into())
			.err_at(|| meta.clone())
	}
}

impl<M: Clone> FunctionalPropertyValue<Id, M> {
	pub fn into_type_at_node_binding(
		self,
		context: &Context<M>,
		id: Id,
		prop: impl Copy + Into<Property>,
	) -> Result<treeldr::FunctionalPropertyValue<treeldr::TId<treeldr::Type>, M>, Error<M>> {
		self.try_unwrap()
			.map_err(|c| c.at_functional_node_property(id, prop))?
			.try_map_borrow_metadata(|p, meta| {
				context.require_type_id(p).map_err(|e| {
					e.at_node_property(id, prop, meta.first().unwrap().value.1.clone())
				})
			})
	}

	pub fn into_datatype_at_node_binding(
		self,
		context: &Context<M>,
		id: Id,
		prop: impl Copy + Into<Property>,
	) -> Result<treeldr::FunctionalPropertyValue<treeldr::TId<treeldr::ty::DataType<M>>, M>, Error<M>>
	{
		self.try_unwrap()
			.map_err(|c| c.at_functional_node_property(id, prop))?
			.try_map_borrow_metadata(|p, meta| {
				context.require_datatype_id(p).map_err(|e| {
					e.at_node_property(id, prop, meta.first().unwrap().value.1.clone())
				})
			})
	}

	pub fn into_required_type_at_node_binding(
		self,
		context: &Context<M>,
		id: Id,
		prop: impl Copy + Into<Property>,
		meta: &M,
	) -> Result<treeldr::RequiredFunctionalPropertyValue<treeldr::TId<treeldr::Type>, M>, Error<M>>
	{
		self.into_type_at_node_binding(context, id, prop)?
			.into_required()
			.ok_or_else(|| error::NodeBindingMissing::new(id, prop).into())
			.err_at(|| meta.clone())
	}

	pub fn into_required_datatype_at_node_binding(
		self,
		context: &Context<M>,
		id: Id,
		prop: impl Copy + Into<Property>,
		meta: &M,
	) -> Result<
		treeldr::RequiredFunctionalPropertyValue<treeldr::TId<treeldr::ty::DataType<M>>, M>,
		Error<M>,
	> {
		self.into_datatype_at_node_binding(context, id, prop)?
			.into_required()
			.ok_or_else(|| error::NodeBindingMissing::new(id, prop).into())
			.err_at(|| meta.clone())
	}

	pub fn into_property_at_node_binding(
		self,
		context: &Context<M>,
		id: Id,
		prop: impl Copy + Into<Property>,
	) -> Result<treeldr::FunctionalPropertyValue<treeldr::TId<treeldr::Property>, M>, Error<M>> {
		self.try_unwrap()
			.map_err(|c| c.at_functional_node_property(id, prop))?
			.try_map_borrow_metadata(|p, meta| {
				context.require_property_id(p).map_err(|e| {
					e.at_node_property(id, prop, meta.first().unwrap().value.1.clone())
				})
			})
	}

	pub fn into_required_property_at_node_binding(
		self,
		context: &Context<M>,
		id: Id,
		prop: impl Copy + Into<Property>,
		meta: &M,
	) -> Result<
		treeldr::RequiredFunctionalPropertyValue<treeldr::TId<treeldr::Property>, M>,
		Error<M>,
	> {
		self.into_property_at_node_binding(context, id, prop)?
			.into_required()
			.ok_or_else(|| error::NodeBindingMissing::new(id, prop).into())
			.err_at(|| meta.clone())
	}

	pub fn into_layout_at_node_binding(
		self,
		context: &Context<M>,
		id: Id,
		prop: impl Copy + Into<Property>,
	) -> Result<treeldr::FunctionalPropertyValue<treeldr::TId<treeldr::Layout>, M>, Error<M>> {
		self.try_unwrap()
			.map_err(|c| c.at_functional_node_property(id, prop))?
			.try_map_borrow_metadata(|p, meta| {
				context.require_layout_id(p).map_err(|e| {
					e.at_node_property(id, prop, meta.first().unwrap().value.1.clone())
				})
			})
	}

	pub fn into_required_layout_at_node_binding(
		self,
		context: &Context<M>,
		id: Id,
		prop: impl Copy + Into<Property>,
		meta: &M,
	) -> Result<treeldr::RequiredFunctionalPropertyValue<treeldr::TId<treeldr::Layout>, M>, Error<M>>
	{
		self.into_layout_at_node_binding(context, id, prop)?
			.into_required()
			.ok_or_else(|| error::NodeBindingMissing::new(id, prop).into())
			.err_at(|| meta.clone())
	}

	pub fn into_list_at_node_binding(
		self,
		context: &Context<M>,
		id: Id,
		prop: impl Copy + Into<Property>,
	) -> Result<treeldr::FunctionalPropertyValue<ListRef<M>, M>, Error<M>> {
		self.try_unwrap()
			.map_err(|c| c.at_functional_node_property(id, prop))?
			.try_map_borrow_metadata(|p, meta| {
				context.require_list(p).map_err(|e| {
					e.at_node_property(id, prop, meta.first().unwrap().value.1.clone())
				})
			})
	}

	pub fn into_required_list_at_node_binding<'l>(
		self,
		context: &'l Context<M>,
		id: Id,
		prop: impl Copy + Into<Property>,
		meta: &M,
	) -> Result<treeldr::RequiredFunctionalPropertyValue<ListRef<'l, M>, M>, Error<M>> {
		self.into_list_at_node_binding(context, id, prop)?
			.into_required()
			.ok_or_else(|| error::NodeBindingMissing::new(id, prop).into())
			.err_at(|| meta.clone())
	}
}

impl<T: Ord, M, N> PartialEq<FunctionalPropertyValue<T, N>> for FunctionalPropertyValue<T, M> {
	fn eq(&self, other: &FunctionalPropertyValue<T, N>) -> bool {
		self.0.iter().any(|s| other.0.contains(s.value.value()))
	}
}

impl<T: Ord, M, N> StrippedPartialEq<FunctionalPropertyValue<T, N>>
	for FunctionalPropertyValue<T, M>
{
	fn stripped_eq(&self, other: &FunctionalPropertyValue<T, N>) -> bool {
		self.0.iter().any(|s| other.0.contains(s.value.value()))
	}
}

impl<T: Ord, M> StrippedEq for FunctionalPropertyValue<T, M> {}

// impl<T: Ord, M, N> StrippedPartialOrd<FunctionalPropertyValue<T, N>> for FunctionalPropertyValue<T, M> {
// 	fn stripped_partial_cmp(&self, other: &FunctionalPropertyValue<T, N>) -> Option<std::cmp::Ordering> {
// 		Some(
// 			self.first()
// 				.map(Meta::into_value)
// 				.cmp(&other.first().map(Meta::into_value)),
// 		)
// 	}
// }

// impl<T: Ord, M> StrippedOrd for FunctionalPropertyValue<T, M> {
// 	fn stripped_cmp(&self, other: &Self) -> std::cmp::Ordering {
// 		self.first()
// 			.map(Meta::into_value)
// 			.cmp(&other.first().map(Meta::into_value))
// 	}
// }

impl<T: Ord, M> From<Meta<T, M>> for FunctionalPropertyValue<T, M> {
	fn from(value: Meta<T, M>) -> Self {
		Self(PropertyValues::from(value))
	}
}

impl<T: Ord, M: Merge> Extend<Meta<T, M>> for FunctionalPropertyValue<T, M> {
	fn extend<I: IntoIterator<Item = Meta<T, M>>>(&mut self, iter: I) {
		self.0.extend(iter)
	}
}

impl<T: Ord, M: Merge> FromIterator<Meta<T, M>> for FunctionalPropertyValue<T, M> {
	fn from_iter<I: IntoIterator<Item = Meta<T, M>>>(iter: I) -> Self {
		Self(PropertyValues::from_iter(iter))
	}
}

impl<T, M> IntoIterator for FunctionalPropertyValue<T, M> {
	type Item = PropertyValue<T, M>;
	type IntoIter = IntoIter<T, M>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

impl<'a, T, M> IntoIterator for &'a FunctionalPropertyValue<T, M> {
	type Item = PropertyValueRef<'a, T, M>;
	type IntoIter = Iter<'a, T, M>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

pub type Iter<'a, T, M> = crate::property_values::non_functional::Iter<'a, T, M>;
pub type IntoIter<T, M> = crate::property_values::non_functional::IntoIter<T, M>;
