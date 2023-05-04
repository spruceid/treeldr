use std::{cmp::Ordering, hash::Hash};

use crate::{metadata::Merge, MetaOption};
use derivative::Derivative;
use locspan::{Meta, StrippedEq, StrippedHash, StrippedOrd, StrippedPartialEq, StrippedPartialOrd};

/// Integer-like constraints.
#[derive(Debug, Clone)]
pub enum Restriction<T> {
	MinInclusive(T),
	MaxInclusive(T),
}

#[derive(Debug, Derivative)]
#[derivative(Clone(bound = ""), Copy(bound = ""))]
pub enum RestrictionRef<'a, T> {
	MinInclusive(&'a T),
	MaxInclusive(&'a T),
}

#[derive(Clone, Debug)]
pub struct Restrictions<T, M> {
	min: MetaOption<T, M>,
	max: MetaOption<T, M>,
}

impl<T: PartialEq, M> StrippedPartialEq for Restrictions<T, M> {
	fn stripped_eq(&self, other: &Self) -> bool {
		self.min.value() == other.min.value() && self.max.value() == other.max.value()
	}
}

impl<T: Eq, M> StrippedEq for Restrictions<T, M> {}

impl<T: PartialOrd, M> StrippedPartialOrd for Restrictions<T, M> {
	fn stripped_partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		match self.min.value().partial_cmp(&other.min.value())? {
			Ordering::Equal => self.max.value().partial_cmp(&other.max.value()),
			cmp => Some(cmp),
		}
	}
}

impl<T: Ord, M> StrippedOrd for Restrictions<T, M> {
	fn stripped_cmp(&self, other: &Self) -> Ordering {
		self.min
			.value()
			.cmp(&other.min.value())
			.then_with(|| self.max.value().cmp(&other.max.value()))
	}
}

impl<T: Hash, M> StrippedHash for Restrictions<T, M> {
	fn stripped_hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.min.value().hash(state);
		self.max.value().hash(state)
	}
}

#[derive(Debug)]
pub struct Conflict<T, M>(pub Restriction<T>, pub Meta<Restriction<T>, M>);

impl<T, M> Default for Restrictions<T, M> {
	fn default() -> Self {
		Self {
			min: MetaOption::default(),
			max: MetaOption::default(),
		}
	}
}

impl<T, M> Restrictions<T, M> {
	pub fn is_restricted(&self) -> bool {
		self.min.is_some() || self.max.is_some()
	}

	pub fn min_with_metadata(&self) -> &MetaOption<T, M> {
		&self.min
	}

	pub fn min(&self) -> Option<&T> {
		self.min.value()
	}

	pub fn insert_min(&mut self, Meta(min, meta): Meta<T, M>) -> Result<(), Meta<Conflict<T, M>, M>>
	where
		M: Clone + Merge,
		T: Clone + PartialOrd,
	{
		if let Some(Meta(max, max_meta)) = self.max.as_ref() {
			if min > *max {
				return Err(Meta(
					Conflict(
						Restriction::MinInclusive(min),
						Meta(Restriction::MaxInclusive(max.clone()), max_meta.clone()),
					),
					meta,
				));
			}
		}

		match self.min.as_mut() {
			Some(Meta(current, current_meta)) => {
				if *current <= min {
					*current = min;
					current_meta.merge_with(meta)
				}
			}
			None => self.min = MetaOption::new(min, meta),
		}

		Ok(())
	}

	pub fn max_with_metadata(&self) -> &MetaOption<T, M> {
		&self.max
	}

	pub fn max(&self) -> Option<&T> {
		self.max.value()
	}

	pub fn insert_max(&mut self, Meta(max, meta): Meta<T, M>) -> Result<(), Meta<Conflict<T, M>, M>>
	where
		M: Clone + Merge,
		T: Clone + PartialOrd,
	{
		if let Some(Meta(min, min_meta)) = self.min.as_ref() {
			if max < *min {
				return Err(Meta(
					Conflict(
						Restriction::MaxInclusive(max),
						Meta(Restriction::MinInclusive(min.clone()), min_meta.clone()),
					),
					meta,
				));
			}
		}

		match self.max.as_mut() {
			Some(Meta(current, current_meta)) => {
				if *current >= max {
					*current = max;
					current_meta.merge_with(meta)
				}
			}
			None => self.max = MetaOption::new(max, meta),
		}

		Ok(())
	}

	pub fn iter(&self) -> Iter<T, M> {
		Iter {
			min: self.min.as_ref(),
			max: self.max.as_ref(),
		}
	}
}

pub struct Iter<'a, T, M> {
	min: Option<&'a Meta<T, M>>,
	max: Option<&'a Meta<T, M>>,
}

impl<'a, T, M> Iterator for Iter<'a, T, M> {
	type Item = Meta<RestrictionRef<'a, T>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.min
			.take()
			.map(|m| m.borrow().map(RestrictionRef::MinInclusive))
			.or_else(|| {
				self.max
					.take()
					.map(|m| m.borrow().map(RestrictionRef::MaxInclusive))
			})
	}
}

impl<'a, T, M> DoubleEndedIterator for Iter<'a, T, M> {
	fn next_back(&mut self) -> Option<Self::Item> {
		self.max
			.take()
			.map(|m| m.borrow().map(RestrictionRef::MaxInclusive))
			.or_else(|| {
				self.min
					.take()
					.map(|m| m.borrow().map(RestrictionRef::MinInclusive))
			})
	}
}
