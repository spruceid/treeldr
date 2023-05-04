use crate::{metadata::Merge, MetaOption};
use derivative::Derivative;
use locspan::{Meta, StrippedEq, StrippedHash, StrippedOrd, StrippedPartialEq, StrippedPartialOrd};
use std::{
	cmp::{Ord, Ordering},
	hash::Hash,
};

pub trait FloatType {
	const INFINITY: Self;
	const NEG_INFINITY: Self;
}

impl FloatType for f32 {
	const INFINITY: Self = f32::INFINITY;
	const NEG_INFINITY: Self = f32::NEG_INFINITY;
}

impl FloatType for f64 {
	const INFINITY: Self = f64::INFINITY;
	const NEG_INFINITY: Self = f64::NEG_INFINITY;
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Min<T> {
	Included(T),
	Excluded(T),
}

impl<T: FloatType> Default for Min<T> {
	fn default() -> Self {
		Min::Included(T::NEG_INFINITY)
	}
}

impl<T> Min<T> {
	pub fn value(&self) -> &T {
		match self {
			Self::Included(v) => v,
			Self::Excluded(v) => v,
		}
	}

	pub fn is_included(&self) -> bool {
		matches!(self, Self::Included(_))
	}

	pub fn is_excluded(&self) -> bool {
		matches!(self, Self::Excluded(_))
	}

	pub fn is_bounded(&self) -> bool
	where
		T: FloatType + PartialEq,
	{
		*self != Self::Included(T::NEG_INFINITY)
	}
}

impl<T: Ord> Ord for Min<T> {
	fn cmp(&self, other: &Self) -> Ordering {
		match self.value().cmp(other.value()) {
			Ordering::Equal => match (self, other) {
				(Self::Included(_), Self::Excluded(_)) => Ordering::Less,
				(Self::Excluded(_), Self::Included(_)) => Ordering::Greater,
				_ => Ordering::Equal,
			},
			other => other,
		}
	}
}

impl<T: PartialOrd> PartialOrd for Min<T> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		match self.value().partial_cmp(other.value())? {
			Ordering::Equal => match (self, other) {
				(Self::Included(_), Self::Excluded(_)) => Some(Ordering::Less),
				(Self::Excluded(_), Self::Included(_)) => Some(Ordering::Greater),
				_ => Some(Ordering::Equal),
			},
			other => Some(other),
		}
	}
}

impl<T: PartialEq> PartialEq<Max<T>> for Min<T> {
	fn eq(&self, other: &Max<T>) -> bool {
		match (self, other) {
			(Self::Included(a), Max::Included(b)) => a.eq(b),
			_ => false,
		}
	}
}

impl<T: PartialOrd> PartialOrd<Max<T>> for Min<T> {
	fn partial_cmp(&self, other: &Max<T>) -> Option<Ordering> {
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
pub enum Max<T> {
	Included(T),
	Excluded(T),
}

impl<T: FloatType> Default for Max<T> {
	fn default() -> Self {
		Max::Included(T::INFINITY)
	}
}

impl<T> Max<T> {
	pub fn value(&self) -> &T {
		match self {
			Self::Included(v) => v,
			Self::Excluded(v) => v,
		}
	}

	pub fn is_included(&self) -> bool {
		matches!(self, Self::Included(_))
	}

	pub fn is_excluded(&self) -> bool {
		matches!(self, Self::Excluded(_))
	}

	pub fn is_bounded(&self) -> bool
	where
		T: FloatType + PartialEq,
	{
		*self != Self::Included(T::INFINITY)
	}
}

impl<T: Ord> Ord for Max<T> {
	fn cmp(&self, other: &Self) -> Ordering {
		match self.value().cmp(other.value()) {
			Ordering::Equal => match (self, other) {
				(Self::Included(_), Self::Excluded(_)) => Ordering::Greater,
				(Self::Excluded(_), Self::Included(_)) => Ordering::Less,
				_ => Ordering::Equal,
			},
			other => other,
		}
	}
}

impl<T: PartialOrd> PartialOrd for Max<T> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		match self.value().partial_cmp(other.value())? {
			Ordering::Equal => match (self, other) {
				(Self::Included(_), Self::Excluded(_)) => Some(Ordering::Greater),
				(Self::Excluded(_), Self::Included(_)) => Some(Ordering::Less),
				_ => Some(Ordering::Equal),
			},
			other => Some(other),
		}
	}
}

impl<T: PartialEq> PartialEq<Min<T>> for Max<T> {
	fn eq(&self, other: &Min<T>) -> bool {
		match (self, other) {
			(Self::Included(a), Min::Included(b)) => a.eq(b),
			_ => false,
		}
	}
}

impl<T: PartialOrd> PartialOrd<Min<T>> for Max<T> {
	fn partial_cmp(&self, other: &Min<T>) -> Option<Ordering> {
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
pub struct Conflict<T, M>(pub Restriction<T>, pub Meta<Restriction<T>, M>);

#[derive(Clone, Debug)]
pub struct Restrictions<T, M> {
	min: MetaOption<Min<T>, M>,
	max: MetaOption<Max<T>, M>,
}

impl<T: FloatType, M> Default for Restrictions<T, M> {
	fn default() -> Self {
		Self {
			min: MetaOption::default(),
			max: MetaOption::default(),
		}
	}
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

impl<T, M> Restrictions<T, M> {
	pub fn is_restricted(&self) -> bool
	where
		T: FloatType + Clone + PartialEq,
	{
		self.min().is_bounded() || self.max().is_bounded()
	}

	pub fn min_with_metadata(&self) -> &MetaOption<Min<T>, M> {
		&self.min
	}

	pub fn min(&self) -> Min<T>
	where
		T: FloatType + Clone,
	{
		self.min.value().cloned().unwrap_or_default()
	}

	pub fn insert_min(
		&mut self,
		Meta(min, meta): Meta<Min<T>, M>,
	) -> Result<(), Meta<Conflict<T, M>, M>>
	where
		M: Clone + Merge,
		T: Clone + PartialOrd,
	{
		if let Some(Meta(max, max_meta)) = self.max.as_ref() {
			if min > *max {
				return Err(Meta(
					Conflict(
						Restriction::Min(min),
						Meta(Restriction::Max(max.clone()), max_meta.clone()),
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

	pub fn max_with_metadata(&self) -> &MetaOption<Max<T>, M> {
		&self.max
	}

	pub fn max(&self) -> Max<T>
	where
		T: FloatType + Clone,
	{
		self.max.value().cloned().unwrap_or_default()
	}

	pub fn insert_max(
		&mut self,
		Meta(max, meta): Meta<Max<T>, M>,
	) -> Result<(), Meta<Conflict<T, M>, M>>
	where
		M: Clone + Merge,
		T: Clone + PartialOrd,
	{
		if let Some(Meta(min, min_meta)) = self.min.as_ref() {
			if max < *min {
				return Err(Meta(
					Conflict(
						Restriction::Max(max),
						Meta(Restriction::Min(min.clone()), min_meta.clone()),
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

	pub fn iter(&self) -> Iter<T, M>
	where
		T: FloatType + PartialEq,
	{
		Iter {
			min: self
				.min
				.as_ref()
				.and_then(|v| if v.is_bounded() { Some(v) } else { None }),
			max: self
				.max
				.as_ref()
				.and_then(|v| if v.is_bounded() { Some(v) } else { None }),
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub enum Restriction<T> {
	Min(Min<T>),
	Max(Max<T>),
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Copy(bound = ""))]
pub enum RestrictionRef<'a, T> {
	Min(&'a Min<T>),
	Max(&'a Max<T>),
}

pub struct Iter<'a, T, M> {
	min: Option<&'a Meta<Min<T>, M>>,
	max: Option<&'a Meta<Max<T>, M>>,
}

impl<'a, T: 'a, M> Iterator for Iter<'a, T, M> {
	type Item = Meta<RestrictionRef<'a, T>, &'a M>;

	fn size_hint(&self) -> (usize, Option<usize>) {
		let len = usize::from(self.min.is_some()) + usize::from(self.max.is_some());
		(len, Some(len))
	}

	fn next(&mut self) -> Option<Self::Item> {
		self.min
			.take()
			.map(|m| m.borrow().map(RestrictionRef::Min))
			.or_else(|| self.max.take().map(|m| m.borrow().map(RestrictionRef::Max)))
	}
}

impl<'a, T: 'a, M> DoubleEndedIterator for Iter<'a, T, M> {
	fn next_back(&mut self) -> Option<Self::Item> {
		self.max
			.take()
			.map(|m| m.borrow().map(RestrictionRef::Max))
			.or_else(|| self.min.take().map(|m| m.borrow().map(RestrictionRef::Min)))
	}
}

impl<'a, T: 'a, M> ExactSizeIterator for Iter<'a, T, M> {}
