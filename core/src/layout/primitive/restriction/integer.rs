use crate::{metadata::Merge, MetaOption};
use locspan::Meta;
use locspan_derive::{
	StrippedEq, StrippedHash, StrippedOrd, StrippedPartialEq, StrippedPartialOrd,
};

#[derive(Debug)]
pub enum Restriction {
	MinInclusive(i64),
	MaxInclusive(i64),
}

#[derive(
	Clone, StrippedPartialEq, StrippedEq, StrippedHash, StrippedPartialOrd, StrippedOrd, Debug,
)]
#[stripped_ignore(M)]
pub struct Restrictions<M> {
	min: MetaOption<i64, M>,
	max: MetaOption<i64, M>,
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
		self.min() != i64::MIN || self.max() != i64::MAX
	}

	pub fn min_with_metadata(&self) -> &MetaOption<i64, M> {
		&self.min
	}

	pub fn min(&self) -> i64 {
		self.min.value().cloned().unwrap_or(i64::MIN)
	}

	pub fn insert_min(&mut self, Meta(min, meta): Meta<i64, M>) -> Result<(), Meta<Conflict<M>, M>>
	where
		M: Clone + Merge,
	{
		if let Some(Meta(max, max_meta)) = self.max.as_ref() {
			if min > *max {
				return Err(Meta(
					Conflict(
						Restriction::MinInclusive(min),
						Meta(Restriction::MaxInclusive(*max), max_meta.clone()),
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

	pub fn max_with_metadata(&self) -> &MetaOption<i64, M> {
		&self.max
	}

	pub fn max(&self) -> i64 {
		self.max.value().cloned().unwrap_or(i64::MAX)
	}

	pub fn insert_max(&mut self, Meta(max, meta): Meta<i64, M>) -> Result<(), Meta<Conflict<M>, M>>
	where
		M: Clone + Merge,
	{
		if let Some(Meta(min, min_meta)) = self.min.as_ref() {
			if max < *min {
				return Err(Meta(
					Conflict(
						Restriction::MaxInclusive(max),
						Meta(Restriction::MinInclusive(*min), min_meta.clone()),
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
				if *v != i64::MIN {
					Some(Meta(*v, m))
				} else {
					None
				}
			}),
			max: self.max.as_ref().and_then(|Meta(v, m)| {
				if *v != i64::MAX {
					Some(Meta(*v, m))
				} else {
					None
				}
			}),
		}
	}
}

pub struct Iter<'a, M> {
	min: Option<Meta<i64, &'a M>>,
	max: Option<Meta<i64, &'a M>>,
}

impl<'a, M> Iterator for Iter<'a, M> {
	type Item = Meta<Restriction, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.min
			.take()
			.map(|m| m.map(Restriction::MinInclusive))
			.or_else(|| self.max.take().map(|m| m.map(Restriction::MaxInclusive)))
	}
}
