use crate::{metadata::Merge, MetaOption};
use locspan::Meta;
use locspan_derive::{
	StrippedEq, StrippedHash, StrippedOrd, StrippedPartialEq, StrippedPartialOrd,
};
use xsd_types::Integer;

#[derive(Debug, Clone)]
pub enum Restriction {
	MinInclusive(Integer),
	MaxInclusive(Integer),
}

#[derive(Debug, Clone, Copy)]
pub enum RestrictionRef<'a> {
	MinInclusive(&'a Integer),
	MaxInclusive(&'a Integer),
}

#[derive(
	Clone, StrippedPartialEq, StrippedEq, StrippedHash, StrippedPartialOrd, StrippedOrd, Debug,
)]
#[locspan(ignore(M))]
pub struct Restrictions<M> {
	#[locspan(unwrap_deref_stripped)]
	min: MetaOption<Integer, M>,

	#[locspan(unwrap_deref_stripped)]
	max: MetaOption<Integer, M>,
}

#[derive(Debug)]
pub struct Conflict<M>(pub Restriction, pub Meta<Restriction, M>);

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
		self.min.is_some() || self.max.is_some()
	}

	pub fn min_with_metadata(&self) -> &MetaOption<Integer, M> {
		&self.min
	}

	pub fn min(&self) -> Option<&Integer> {
		self.min.value()
	}

	pub fn insert_min(
		&mut self,
		Meta(min, meta): Meta<Integer, M>,
	) -> Result<(), Meta<Conflict<M>, M>>
	where
		M: Clone + Merge,
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

	pub fn max_with_metadata(&self) -> &MetaOption<Integer, M> {
		&self.max
	}

	pub fn max(&self) -> Option<&Integer> {
		self.max.value()
	}

	pub fn insert_max(
		&mut self,
		Meta(max, meta): Meta<Integer, M>,
	) -> Result<(), Meta<Conflict<M>, M>>
	where
		M: Clone + Merge,
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

	pub fn iter(&self) -> Iter<M> {
		Iter {
			min: self.min.as_ref(),
			max: self.max.as_ref(),
		}
	}
}

pub struct Iter<'a, M> {
	min: Option<&'a Meta<Integer, M>>,
	max: Option<&'a Meta<Integer, M>>,
}

impl<'a, M> Iterator for Iter<'a, M> {
	type Item = Meta<RestrictionRef<'a>, &'a M>;

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

impl<'a, M> DoubleEndedIterator for Iter<'a, M> {
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
