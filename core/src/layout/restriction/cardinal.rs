use crate::{metadata::Merge, MetaOption};
use locspan::Meta;
use locspan_derive::{StrippedEq, StrippedPartialEq};
use super::Property;

/// Cardinal restriction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Restriction {
	Min(u64),
	Max(u64),
}

impl Restriction {
	pub fn as_binding(&self) -> Binding {
		match self {
			Self::Min(v) => Binding::Min(*v),
			Self::Max(v) => Binding::Max(*v)
		}
	}
}

/// Conflicting cardinal restrictions.
#[derive(Debug)]
pub struct Conflict<M>(pub Restriction, pub Meta<Restriction, M>);

/// Cardinal restrictions.
#[derive(Clone, Debug, StrippedPartialEq, StrippedEq)]
#[locspan(ignore(M))]
pub struct Restrictions<M> {
	min: MetaOption<u64, M>,
	max: MetaOption<u64, M>,
}

impl<M> Default for Restrictions<M> {
	fn default() -> Self {
		Self {
			min: MetaOption::default(),
			max: MetaOption::default(),
		}
	}
}

impl<M> Restrictions<M> {
	pub fn is_included_in(&self, other: &Self) -> bool {
		self.min.value() >= other.min.value() && self.max.value() <= other.max.value()
	}

	pub fn insert(
		&mut self,
		Meta(restriction, meta): Meta<Restriction, M>,
	) -> Result<(), Meta<Conflict<M>, M>>
	where
		M: Clone + Merge,
	{
		match restriction {
			Restriction::Min(m) => self.insert_min(Meta(m, meta)),
			Restriction::Max(m) => self.insert_max(Meta(m, meta)),
		}
	}

	pub fn insert_min(&mut self, Meta(min, meta): Meta<u64, M>) -> Result<(), Meta<Conflict<M>, M>>
	where
		M: Clone + Merge,
	{
		if let Some(Meta(max, max_meta)) = self.max.as_ref() {
			if min > *max {
				return Err(Meta(
					Conflict(
						Restriction::Min(min),
						Meta(Restriction::Max(*max), max_meta.clone()),
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

	pub fn insert_max(&mut self, Meta(max, meta): Meta<u64, M>) -> Result<(), Meta<Conflict<M>, M>>
	where
		M: Clone + Merge,
	{
		if let Some(Meta(min, min_meta)) = self.min.as_ref() {
			if max < *min {
				return Err(Meta(
					Conflict(
						Restriction::Max(max),
						Meta(Restriction::Min(*min), min_meta.clone()),
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

	pub fn unify(&mut self, other: Self) -> Result<(), Meta<Conflict<M>, M>>
	where
		M: Clone + Merge,
	{
		if let Some(min) = other.min.unwrap() {
			self.insert_min(min)?
		}

		if let Some(max) = other.max.unwrap() {
			self.insert_max(max)?
		}

		Ok(())
	}

	pub fn min(&self) -> u64 {
		self.min.value().cloned().unwrap_or(0)
	}

	pub fn max(&self) -> u64 {
		self.max.value().cloned().unwrap_or(u64::MAX)
	}

	pub fn min_with_metadata(&self) -> &MetaOption<u64, M> {
		&self.min
	}

	pub fn max_with_metadata(&self) -> &MetaOption<u64, M> {
		&self.max
	}

	/// Checks if the cardinal is restricted.
	pub fn is_restricted(&self) -> bool {
		self.min() != 0 || self.max() != u64::MAX
	}

	/// Checks if the required cardinal is at least 1.
	pub fn is_required(&self) -> bool {
		self.min() != 0
	}
}

pub enum Binding {
	Min(u64),
	Max(u64),
}

impl Binding {
	pub fn property(&self) -> Property {
		match self {
			Self::Min(_) => Property::MinCardinality,
			Self::Max(_) => Property::MaxCardinality
		}
	}
}