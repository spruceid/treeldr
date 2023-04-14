use derivative::Derivative;
use locspan::Meta;

use crate::metadata::Merge;
use crate::{Property, TId};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use super::{restriction::Contradiction, Restriction, Restrictions};

#[derive(Debug, Clone)]
struct PropertyData<M> {
	metadata: M,
	restrictions: Restrictions<M>,
}

impl<M> PropertyData<M> {
	pub fn restrict(&mut self, restriction: Meta<Restriction, M>) -> Result<(), Contradiction>
	where
		M: Clone + Merge,
	{
		self.restrictions.restrict(restriction)
	}
}

/// Type properties.
#[derive(Debug, Clone)]
pub struct Properties<M> {
	/// Included properties, with their eventual restrictions.
	included: HashMap<TId<Property>, PropertyData<M>>,

	/// Excluded properties.
	///
	/// If `None`, then all the properties not
	/// in `included` are excluded.
	excluded: Option<HashMap<TId<Property>, M>>,
}

impl<M> Properties<M> {
	pub fn none() -> Self {
		Self {
			included: HashMap::new(),
			excluded: None,
		}
	}

	pub fn all() -> Self {
		Self {
			included: HashMap::new(),
			excluded: Some(HashMap::new()),
		}
	}

	pub fn included(&self) -> IncludedProperties<M> {
		IncludedProperties {
			inner: self.included.iter(),
		}
	}

	pub fn iter(&self) -> Iter<M> {
		Iter {
			included: IncludedProperties {
				inner: self.included.iter(),
			},
			excluded: self.excluded.as_ref().map(|excluded| ExcludedProperties {
				inner: excluded.iter(),
			}),
		}
	}
}

impl<M> Default for Properties<M> {
	fn default() -> Self {
		Self::none()
	}
}

impl<M> Properties<M> {
	pub fn insert(
		&mut self,
		prop: TId<Property>,
		restrictions: Option<Restrictions<M>>,
		metadata: M,
	) where
		M: Clone + Merge,
	{
		match self.included.entry(prop) {
			Entry::Occupied(mut entry) => {
				let data = entry.get_mut();
				data.metadata.merge_with(metadata);
				match restrictions {
					Some(restrictions) => {
						data.restrictions = data.restrictions.union_with(&restrictions)
					}
					None => data.restrictions.clear(),
				}
			}
			Entry::Vacant(entry) => {
				entry.insert(PropertyData {
					restrictions: restrictions.unwrap_or_default(),
					metadata,
				});
			}
		}

		if let Some(excluded) = &mut self.excluded {
			excluded.remove(&prop);
		}
	}

	pub fn remove(&mut self, prop: TId<Property>, metadata: M) {
		self.included.remove(&prop);

		if let Some(excluded) = &mut self.excluded {
			excluded.insert(prop, metadata);
		}
	}

	/// Further restrict `prop` if it is included in this set of properties.
	pub fn restrict(
		&mut self,
		prop: TId<Property>,
		Meta(restriction, metadata): Meta<Restriction, M>,
	) -> Result<(), Contradiction>
	where
		M: Clone + Merge,
	{
		if let Some(data) = self.included.get_mut(&prop) {
			data.restrict(Meta(restriction, metadata.clone()))?;
			data.metadata.merge_with(metadata);
		}

		Ok(())
	}

	pub fn unite_with(&mut self, other: &Self)
	where
		M: Clone + Merge,
	{
		for (&prop, data) in &other.included {
			self.insert(prop, Some(data.restrictions.clone()), data.metadata.clone());
		}

		if let (Some(excluded), Some(other_excluded)) = (&mut self.excluded, &other.excluded) {
			for prop in other_excluded.keys() {
				excluded.remove(prop);
			}
		}
	}

	pub fn union_with(&self, other: &Self) -> Self
	where
		M: Clone + Merge,
	{
		let mut result = self.clone();
		result.unite_with(other);
		result
	}

	pub fn intersect_with(&mut self, other: &Self) -> Result<(), Contradiction>
	where
		M: Clone + Merge,
	{
		self.excluded = match (self.excluded.take(), &other.excluded) {
			(Some(mut excluded), Some(other_excluded)) => {
				for (prop, cause) in other_excluded {
					match excluded.entry(*prop) {
						Entry::Occupied(mut entry) => entry.get_mut().merge_with(cause.clone()),
						Entry::Vacant(entry) => {
							entry.insert(cause.clone());
						}
					}
				}
				Some(excluded)
			}
			_ => None,
		};

		let included = std::mem::take(&mut self.included);
		for (prop, data) in included {
			match other.included.get(&prop) {
				Some(other_data) => {
					let data = PropertyData {
						metadata: data
							.metadata
							.clone()
							.merged_with(other_data.metadata.clone()),
						restrictions: data
							.restrictions
							.intersection_with(&other_data.restrictions)?,
					};

					self.included.insert(prop, data);
				}
				None => {
					if let Some(excluded) = &mut self.excluded {
						excluded.insert(prop, data.metadata.clone());
					}
				}
			}
		}

		if let Some(excluded) = &mut self.excluded {
			for (prop, data) in &other.included {
				if !self.included.contains_key(prop) {
					excluded.insert(*prop, data.metadata.clone());
				}
			}
		}

		Ok(())
	}

	pub fn intersection_with(&self, other: &Self) -> Result<Self, Contradiction>
	where
		M: Clone + Merge,
	{
		let mut result = self.clone();
		result.intersect_with(other)?;
		Ok(result)
	}
}

impl<'a, M> IntoIterator for &'a Properties<M> {
	type Item = PseudoProperty<'a, M>;
	type IntoIter = Iter<'a, M>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

pub struct ExcludedProperties<'a, M> {
	inner: std::collections::hash_map::Iter<'a, TId<Property>, M>,
}

impl<'a, M> Iterator for ExcludedProperties<'a, M> {
	type Item = (TId<Property>, &'a M);

	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next().map(|(prop, causes)| (*prop, causes))
	}
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Copy(bound = ""))]
pub struct RestrictedProperty<'a, M> {
	prop: TId<Property>,
	restrictions: &'a Restrictions<M>,
	causes: &'a M,
}

impl<'a, M> RestrictedProperty<'a, M> {
	pub fn property(&self) -> TId<Property> {
		self.prop
	}

	pub fn restrictions(&self) -> &'a Restrictions<M> {
		self.restrictions
	}

	pub fn causes(&self) -> &'a M {
		self.causes
	}
}

pub enum PseudoProperty<'a, M> {
	Property(RestrictedProperty<'a, M>),
	Complement(ExcludedProperties<'a, M>),
}

pub struct Iter<'a, M> {
	included: IncludedProperties<'a, M>,
	excluded: Option<ExcludedProperties<'a, M>>,
}

impl<'a, M> Iterator for Iter<'a, M> {
	type Item = PseudoProperty<'a, M>;

	fn next(&mut self) -> Option<Self::Item> {
		match self.included.next() {
			Some(item) => Some(PseudoProperty::Property(item)),
			None => self.excluded.take().map(PseudoProperty::Complement),
		}
	}
}

pub struct IncludedProperties<'a, M> {
	inner: std::collections::hash_map::Iter<'a, TId<Property>, PropertyData<M>>,
}

impl<'a, M> Iterator for IncludedProperties<'a, M> {
	type Item = RestrictedProperty<'a, M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next().map(|(prop, data)| RestrictedProperty {
			prop: *prop,
			restrictions: &data.restrictions,
			causes: &data.metadata,
		})
	}
}
