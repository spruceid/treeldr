use crate::{layout::primitive::RestrictionSet, metadata::Merge, ty::data::RegExp, MetaOption};
use derivative::Derivative;
use locspan::Meta;
use xsd_types::NonNegativeInteger;

use super::RestrictionsTemplate;

pub struct Template;

impl<T> RestrictionsTemplate<T> for Template {
	type Ref<'a> = RestrictionRef<'a> where T: 'a;
	type Set<M> = Restrictions<M>;
	type Iter<'a, M> = Iter<'a, M> where T: 'a, M: 'a;
}

#[derive(Debug)]
pub enum Restriction {
	MinLength(NonNegativeInteger),
	MaxLength(NonNegativeInteger),
	MinGrapheme(NonNegativeInteger),
	MaxGrapheme(NonNegativeInteger),
	Pattern(RegExp),
}

#[derive(Debug)]
pub struct Conflict<M>(pub Restriction, pub Meta<Restriction, M>);

#[derive(Clone, Debug)]
pub struct Restrictions<M> {
	len_min: MetaOption<NonNegativeInteger, M>,
	len_max: MetaOption<NonNegativeInteger, M>,
	grapheme_min: MetaOption<NonNegativeInteger, M>,
	grapheme_max: MetaOption<NonNegativeInteger, M>,
	pattern: MetaOption<RegExp, M>,
}

impl<M> Default for Restrictions<M> {
	fn default() -> Self {
		Self {
			len_min: MetaOption::default(),
			len_max: MetaOption::default(),
			grapheme_min: MetaOption::default(),
			grapheme_max: MetaOption::default(),
			pattern: MetaOption::default(),
		}
	}
}

impl<M> Restrictions<M> {
	pub fn is_len_bounded(&self) -> bool {
		self.len_max.is_some() || !self.len_min().is_zero()
	}

	pub fn is_restricted(&self) -> bool {
		self.pattern.is_some() || self.is_len_bounded()
	}

	pub fn len_min_with_metadata(&self) -> &MetaOption<NonNegativeInteger, M> {
		&self.len_min
	}

	pub fn len_min(&self) -> NonNegativeInteger {
		self.len_min
			.value()
			.cloned()
			.unwrap_or_else(NonNegativeInteger::zero)
	}

	pub fn insert_len_min(
		&mut self,
		Meta(min, meta): Meta<NonNegativeInteger, M>,
	) -> Result<(), Meta<Conflict<M>, M>>
	where
		M: Clone + Merge,
	{
		if let Some(Meta(max, max_meta)) = self.len_max.as_ref() {
			if min > *max {
				return Err(Meta(
					Conflict(
						Restriction::MinLength(min),
						Meta(Restriction::MaxLength(max.clone()), max_meta.clone()),
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

	pub fn len_max_with_metadata(&self) -> &MetaOption<NonNegativeInteger, M> {
		&self.len_max
	}

	pub fn len_max(&self) -> Option<&NonNegativeInteger> {
		self.len_max.value()
	}

	pub fn insert_len_max(
		&mut self,
		Meta(max, meta): Meta<NonNegativeInteger, M>,
	) -> Result<(), Meta<Conflict<M>, M>>
	where
		M: Clone + Merge,
	{
		if let Some(Meta(min, min_meta)) = self.len_min.as_ref() {
			if max < *min {
				return Err(Meta(
					Conflict(
						Restriction::MaxLength(max),
						Meta(Restriction::MinLength(min.clone()), min_meta.clone()),
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

	pub fn grapheme_min_with_metadata(&self) -> &MetaOption<NonNegativeInteger, M> {
		&self.grapheme_min
	}

	pub fn grapheme_min(&self) -> NonNegativeInteger {
		self.len_min
			.value()
			.cloned()
			.unwrap_or_else(NonNegativeInteger::zero)
	}

	pub fn insert_grapheme_min(
		&mut self,
		Meta(min, meta): Meta<NonNegativeInteger, M>,
	) -> Result<(), Meta<Conflict<M>, M>>
	where
		M: Clone + Merge,
	{
		if let Some(Meta(max, max_meta)) = self.grapheme_max.as_ref() {
			if min > *max {
				return Err(Meta(
					Conflict(
						Restriction::MinLength(min),
						Meta(Restriction::MaxLength(max.clone()), max_meta.clone()),
					),
					meta,
				));
			}
		}

		match self.grapheme_min.as_mut() {
			Some(Meta(current, current_meta)) => {
				if *current <= min {
					*current = min;
					current_meta.merge_with(meta)
				}
			}
			None => self.grapheme_min = MetaOption::new(min, meta),
		}

		Ok(())
	}

	pub fn grapheme_max_with_metadata(&self) -> &MetaOption<NonNegativeInteger, M> {
		&self.grapheme_max
	}

	pub fn grapheme_max(&self) -> Option<&NonNegativeInteger> {
		self.grapheme_max.value()
	}

	pub fn insert_grapheme_max(
		&mut self,
		Meta(max, meta): Meta<NonNegativeInteger, M>,
	) -> Result<(), Meta<Conflict<M>, M>>
	where
		M: Clone + Merge,
	{
		if let Some(Meta(min, min_meta)) = self.grapheme_min.as_ref() {
			if max < *min {
				return Err(Meta(
					Conflict(
						Restriction::MaxGrapheme(max),
						Meta(Restriction::MinGrapheme(min.clone()), min_meta.clone()),
					),
					meta,
				));
			}
		}

		match self.grapheme_max.as_mut() {
			Some(Meta(current, current_meta)) => {
				if *current >= max {
					*current = max;
					current_meta.merge_with(meta)
				}
			}
			None => self.grapheme_max = MetaOption::new(max, meta),
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
			len_min: self.len_min.as_ref().filter(|v| !v.is_zero()),
			len_max: self.len_max.as_ref(),
			grapheme_min: self.grapheme_min.as_ref().filter(|v| !v.is_zero()),
			grapheme_max: self.grapheme_max.as_ref(),
			pattern: self.pattern.as_ref(),
		}
	}
}

impl<M> RestrictionSet for Restrictions<M> {
	fn is_restricted(&self) -> bool {
		self.is_restricted()
	}
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Copy(bound = ""))]
pub enum RestrictionRef<'a> {
	MinLength(&'a NonNegativeInteger),
	MaxLength(&'a NonNegativeInteger),
	MinGrapheme(&'a NonNegativeInteger),
	MaxGrapheme(&'a NonNegativeInteger),
	Pattern(&'a RegExp),
}

pub struct Iter<'a, M> {
	len_min: Option<&'a Meta<NonNegativeInteger, M>>,
	len_max: Option<&'a Meta<NonNegativeInteger, M>>,
	grapheme_min: Option<&'a Meta<NonNegativeInteger, M>>,
	grapheme_max: Option<&'a Meta<NonNegativeInteger, M>>,
	pattern: Option<&'a Meta<RegExp, M>>,
}

impl<'a, M> Iterator for Iter<'a, M> {
	type Item = Meta<RestrictionRef<'a>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.len_min
			.take()
			.map(|m| m.borrow().map(RestrictionRef::MinLength))
			.or_else(|| {
				self.len_max
					.take()
					.map(|m| m.borrow().map(RestrictionRef::MaxLength))
			})
			.or_else(|| {
				self.grapheme_min
					.take()
					.map(|m| m.borrow().map(RestrictionRef::MinGrapheme))
			})
			.or_else(|| {
				self.grapheme_max
					.take()
					.map(|m| m.borrow().map(RestrictionRef::MaxGrapheme))
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
					.map(|m| m.borrow().map(RestrictionRef::MaxLength))
			})
			.or_else(|| {
				self.len_min
					.take()
					.map(|m| m.borrow().map(RestrictionRef::MinLength))
			})
			.or_else(|| {
				self.grapheme_min
					.take()
					.map(|m| m.borrow().map(RestrictionRef::MinGrapheme))
			})
			.or_else(|| {
				self.grapheme_max
					.take()
					.map(|m| m.borrow().map(RestrictionRef::MaxGrapheme))
			})
	}
}