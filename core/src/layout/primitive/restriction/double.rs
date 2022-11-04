use crate::{metadata::Merge, value::Double, MetaOption};
use locspan::Meta;
use locspan_derive::{
	StrippedEq, StrippedHash, StrippedOrd, StrippedPartialEq, StrippedPartialOrd,
};
use std::cmp::{Ord, Ordering};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Min {
	Included(Double),
	Excluded(Double),
}

impl Default for Min {
	fn default() -> Self {
		Min::Included(Double::NEG_INFINITY)
	}
}

impl Min {
	pub fn value(&self) -> Double {
		match self {
			Self::Included(v) => *v,
			Self::Excluded(v) => *v,
		}
	}

	pub fn is_included(&self) -> bool {
		matches!(self, Self::Included(_))
	}

	pub fn is_excluded(&self) -> bool {
		matches!(self, Self::Excluded(_))
	}

	pub fn is_bounded(&self) -> bool {
		*self != Self::Included(Double::NEG_INFINITY)
	}
}

impl Ord for Min {
	fn cmp(&self, other: &Self) -> Ordering {
		match self.value().cmp(&other.value()) {
			Ordering::Equal => match (self, other) {
				(Self::Included(_), Self::Excluded(_)) => Ordering::Less,
				(Self::Excluded(_), Self::Included(_)) => Ordering::Greater,
				_ => Ordering::Equal,
			},
			other => other,
		}
	}
}

impl PartialOrd for Min {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl PartialEq<Max> for Min {
	fn eq(&self, other: &Max) -> bool {
		match (self, other) {
			(Self::Included(a), Max::Included(b)) => a.eq(b),
			_ => false,
		}
	}
}

impl PartialOrd<Max> for Min {
	fn partial_cmp(&self, other: &Max) -> Option<Ordering> {
		match (self, other) {
			(Self::Included(a), Max::Included(b)) => a.partial_cmp(b),
			(Self::Included(a) | Self::Excluded(a), Max::Included(b) | Max::Excluded(b)) => {
				match a.partial_cmp(b) {
					Some(Ordering::Equal) => Some(Ordering::Less),
					ordering => ordering,
				}
			}
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Max {
	Included(Double),
	Excluded(Double),
}

impl Default for Max {
	fn default() -> Self {
		Max::Included(Double::INFINITY)
	}
}

impl Max {
	pub fn value(&self) -> Double {
		match self {
			Self::Included(v) => *v,
			Self::Excluded(v) => *v,
		}
	}

	pub fn is_included(&self) -> bool {
		matches!(self, Self::Included(_))
	}

	pub fn is_excluded(&self) -> bool {
		matches!(self, Self::Excluded(_))
	}

	pub fn is_bounded(&self) -> bool {
		*self != Self::Included(Double::INFINITY)
	}
}

impl Ord for Max {
	fn cmp(&self, other: &Self) -> Ordering {
		match self.value().cmp(&other.value()) {
			Ordering::Equal => match (self, other) {
				(Self::Included(_), Self::Excluded(_)) => Ordering::Greater,
				(Self::Excluded(_), Self::Included(_)) => Ordering::Less,
				_ => Ordering::Equal,
			},
			other => other,
		}
	}
}

impl PartialOrd for Max {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl PartialEq<Min> for Max {
	fn eq(&self, other: &Min) -> bool {
		match (self, other) {
			(Self::Included(a), Min::Included(b)) => a.eq(b),
			_ => false,
		}
	}
}

impl PartialOrd<Min> for Max {
	fn partial_cmp(&self, other: &Min) -> Option<Ordering> {
		match (self, other) {
			(Self::Included(a), Min::Included(b)) => a.partial_cmp(b),
			(Self::Included(a) | Self::Excluded(a), Min::Included(b) | Min::Excluded(b)) => {
				match a.partial_cmp(b) {
					Some(Ordering::Equal) => Some(Ordering::Greater),
					ordering => ordering,
				}
			}
		}
	}
}

#[derive(Debug)]
pub struct Conflict<M>(pub Restriction, pub Meta<Restriction, M>);

#[derive(
	Clone, StrippedPartialEq, StrippedEq, StrippedHash, StrippedPartialOrd, StrippedOrd, Debug,
)]
#[locspan(ignore(M))]
pub struct Restrictions<M> {
	#[locspan(unwrap_deref_stripped)]
	min: MetaOption<Min, M>,

	#[locspan(unwrap_deref_stripped)]
	max: MetaOption<Max, M>,
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
	pub fn is_restricted(&self) -> bool {
		self.min().is_bounded() || self.max().is_bounded()
	}

	pub fn min_with_metadata(&self) -> &MetaOption<Min, M> {
		&self.min
	}

	pub fn min(&self) -> Min {
		self.min.value().cloned().unwrap_or_default()
	}

	pub fn insert_min(&mut self, Meta(min, meta): Meta<Min, M>) -> Result<(), Meta<Conflict<M>, M>>
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

	pub fn max_with_metadata(&self) -> &MetaOption<Max, M> {
		&self.max
	}

	pub fn max(&self) -> Max {
		self.max.value().cloned().unwrap_or_default()
	}

	pub fn insert_max(&mut self, Meta(max, meta): Meta<Max, M>) -> Result<(), Meta<Conflict<M>, M>>
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

	pub fn iter(&self) -> Iter<M> {
		Iter {
			min: self.min.as_ref().and_then(|Meta(v, m)| {
				if v.is_bounded() {
					Some(Meta(*v, m))
				} else {
					None
				}
			}),
			max: self.max.as_ref().and_then(|Meta(v, m)| {
				if v.is_bounded() {
					Some(Meta(*v, m))
				} else {
					None
				}
			}),
		}
	}
}

#[derive(Debug)]
pub enum Restriction {
	Min(Min),
	Max(Max),
}

pub struct Iter<'a, M> {
	min: Option<Meta<Min, &'a M>>,
	max: Option<Meta<Max, &'a M>>,
}

impl<'a, M> Iterator for Iter<'a, M> {
	type Item = Meta<Restriction, &'a M>;

	fn size_hint(&self) -> (usize, Option<usize>) {
		let len = usize::from(self.min.is_some()) + usize::from(self.max.is_some());
		(len, Some(len))
	}

	fn next(&mut self) -> Option<Self::Item> {
		self.min
			.take()
			.map(|m| m.map(Restriction::Min))
			.or_else(|| self.max.take().map(|m| m.map(Restriction::Max)))
	}
}

impl<'a, M> ExactSizeIterator for Iter<'a, M> {}
