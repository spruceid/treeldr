use crate::{metadata::Merge, ty::data::RegExp, MetaOption};
use locspan::Meta;

#[derive(Debug)]
pub enum Restriction {
	MinLength(u64),
	MaxLength(u64),
	Pattern(RegExp),
}

#[derive(Debug)]
pub struct Conflict<M>(pub Restriction, pub Meta<Restriction, M>);

#[derive(Clone, Debug)]
pub struct Restrictions<M> {
	len_min: MetaOption<u64, M>,
	len_max: MetaOption<u64, M>,
	pattern: MetaOption<RegExp, M>,
}

impl<M> Default for Restrictions<M> {
	fn default() -> Self {
		Self {
			len_min: MetaOption::default(),
			len_max: MetaOption::default(),
			pattern: MetaOption::default(),
		}
	}
}

impl<M> Restrictions<M> {
	pub fn is_len_bounded(&self) -> bool {
		self.len_min() > 0 || self.len_max() < u64::MAX
	}

	pub fn is_restricted(&self) -> bool {
		self.pattern.is_some() || self.is_len_bounded()
	}

	pub fn len_min_with_metadata(&self) -> &MetaOption<u64, M> {
		&self.len_min
	}

	pub fn len_min(&self) -> u64 {
		self.len_min.value().cloned().unwrap_or(u64::MIN)
	}

	pub fn insert_len_min(
		&mut self,
		Meta(min, meta): Meta<u64, M>,
	) -> Result<(), Meta<Conflict<M>, M>>
	where
		M: Clone + Merge,
	{
		if let Some(Meta(max, max_meta)) = self.len_max.as_ref() {
			if min > *max {
				return Err(Meta(
					Conflict(
						Restriction::MinLength(min),
						Meta(Restriction::MaxLength(*max), max_meta.clone()),
					),
					meta,
				));
			}
		}

		match self.len_min.as_mut() {
			Some(Meta(current, current_meta)) => {
				if *current <= min {
					*current = min;
					current_meta.merge_with(meta)
				}
			}
			None => self.len_min = MetaOption::new(min, meta),
		}

		Ok(())
	}

	pub fn len_max_with_metadata(&self) -> &MetaOption<u64, M> {
		&self.len_max
	}

	pub fn len_max(&self) -> u64 {
		self.len_max.value().cloned().unwrap_or(u64::MAX)
	}

	pub fn insert_len_max(
		&mut self,
		Meta(max, meta): Meta<u64, M>,
	) -> Result<(), Meta<Conflict<M>, M>>
	where
		M: Clone + Merge,
	{
		if let Some(Meta(min, min_meta)) = self.len_min.as_ref() {
			if max < *min {
				return Err(Meta(
					Conflict(
						Restriction::MaxLength(max),
						Meta(Restriction::MinLength(*min), min_meta.clone()),
					),
					meta,
				));
			}
		}

		match self.len_max.as_mut() {
			Some(Meta(current, current_meta)) => {
				if *current >= max {
					*current = max;
					current_meta.merge_with(meta)
				}
			}
			None => self.len_max = MetaOption::new(max, meta),
		}

		Ok(())
	}

	pub fn pattern(&self) -> Option<&Meta<RegExp, M>> {
		self.pattern.as_ref()
	}

	pub fn insert_pattern(&mut self, Meta(regexp, meta): Meta<RegExp, M>)
	where
		M: Merge,
	{
		match self.pattern.as_mut() {
			Some(Meta(current, current_meta)) => {
				if *current == regexp {
					current_meta.merge_with(meta)
				} else {
					todo!("intersect patterns")
				}
			}
			None => self.pattern = MetaOption::new(regexp, meta),
		}
	}

	pub fn is_simple_regexp(&self) -> bool {
		self.pattern.is_some() && !self.is_len_bounded()
	}

	pub fn as_simple_regexp(&self) -> Option<&Meta<RegExp, M>> {
		if self.is_len_bounded() {
			None
		} else {
			self.pattern.as_ref()
		}
	}

	pub fn iter(&self) -> Iter<M> {
		Iter {
			len_min: self.len_min.as_ref().and_then(|Meta(v, m)| {
				if *v != u64::MIN {
					Some(Meta(*v, m))
				} else {
					None
				}
			}),
			len_max: self.len_max.as_ref().and_then(|Meta(v, m)| {
				if *v != u64::MAX {
					Some(Meta(*v, m))
				} else {
					None
				}
			}),
			pattern: self.pattern.as_ref(),
		}
	}
}

#[derive(Clone, Copy)]
pub enum RestrictionRef<'a> {
	MinLength(u64),
	MaxLength(u64),
	Pattern(&'a RegExp),
}

pub struct Iter<'a, M> {
	len_min: Option<Meta<u64, &'a M>>,
	len_max: Option<Meta<u64, &'a M>>,
	pattern: Option<&'a Meta<RegExp, M>>,
}

impl<'a, M> Iterator for Iter<'a, M> {
	type Item = Meta<RestrictionRef<'a>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.len_min
			.take()
			.map(|m| m.map(RestrictionRef::MinLength))
			.or_else(|| {
				self.len_max
					.take()
					.map(|m| m.map(RestrictionRef::MaxLength))
			})
			.or_else(|| {
				self.pattern
					.take()
					.map(|m| m.borrow().map(RestrictionRef::Pattern))
			})
	}
}

impl<'a, M> DoubleEndedIterator for Iter<'a, M> {
	fn next_back(&mut self) -> Option<Self::Item> {
		self.pattern
			.take()
			.map(|m| m.borrow().map(RestrictionRef::Pattern))
			.or_else(|| {
				self.len_max
					.take()
					.map(|m| m.map(RestrictionRef::MaxLength))
			})
			.or_else(|| {
				self.len_min
					.take()
					.map(|m| m.map(RestrictionRef::MinLength))
			})
	}
}
