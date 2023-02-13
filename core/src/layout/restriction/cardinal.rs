use super::Property;
use crate::{metadata::Merge, MetaOption};
use derivative::Derivative;
use locspan::Meta;
use locspan_derive::{StrippedEq, StrippedPartialEq};
use xsd_types::NonNegativeInteger;

/// Cardinal restriction.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Restriction {
	Min(NonNegativeInteger),
	Max(NonNegativeInteger),
}

impl Restriction {
	pub fn as_binding_ref(&self) -> BindingRef {
		match self {
			Self::Min(v) => BindingRef::Min(v),
			Self::Max(v) => BindingRef::Max(v),
		}
	}
}

/// Cardinal restriction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RestrictionRef<'a> {
	Min(&'a NonNegativeInteger),
	Max(&'a NonNegativeInteger),
}

impl<'a> RestrictionRef<'a> {
	pub fn as_binding_ref(&self) -> BindingRef<'a> {
		match self {
			Self::Min(v) => BindingRef::Min(v),
			Self::Max(v) => BindingRef::Max(v),
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
	#[locspan(unwrap_deref_stripped)]
	min: MetaOption<NonNegativeInteger, M>,

	#[locspan(unwrap_deref_stripped)]
	max: MetaOption<NonNegativeInteger, M>,
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

	#[allow(clippy::should_implement_trait)]
	pub fn into_iter(self) -> impl DoubleEndedIterator<Item = Meta<Restriction, M>> {
		self.min
			.unwrap()
			.into_iter()
			.map(|m| m.map(Restriction::Min))
			.chain(
				self.max
					.unwrap()
					.into_iter()
					.map(|m| m.map(Restriction::Max)),
			)
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

	pub fn insert_min(
		&mut self,
		Meta(min, meta): Meta<NonNegativeInteger, M>,
	) -> Result<(), Meta<Conflict<M>, M>>
	where
		M: Clone + Merge,
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

	pub fn insert_max(
		&mut self,
		Meta(max, meta): Meta<NonNegativeInteger, M>,
	) -> Result<(), Meta<Conflict<M>, M>>
	where
		M: Clone + Merge,
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

	pub fn min(&self) -> NonNegativeInteger {
		self.min
			.value()
			.cloned()
			.unwrap_or_else(NonNegativeInteger::zero)
	}

	pub fn max(&self) -> Option<&NonNegativeInteger> {
		self.max.value()
	}

	pub fn min_with_metadata(&self) -> &MetaOption<NonNegativeInteger, M> {
		&self.min
	}

	pub fn max_with_metadata(&self) -> &MetaOption<NonNegativeInteger, M> {
		&self.max
	}

	/// Checks if the cardinal is restricted.
	pub fn is_restricted(&self) -> bool {
		self.max.is_some() || !self.min().is_zero()
	}

	/// Checks if the required cardinal is at least 1.
	pub fn is_required(&self) -> bool {
		!self.min().is_zero()
	}

	pub fn iter(&self) -> RestrictionsIter<M> {
		RestrictionsIter {
			min: self.min.as_ref(),
			max: self.max.as_ref(),
		}
	}
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct RestrictionsIter<'a, M> {
	min: Option<&'a Meta<NonNegativeInteger, M>>,
	max: Option<&'a Meta<NonNegativeInteger, M>>,
}

impl<'a, M> Iterator for RestrictionsIter<'a, M> {
	type Item = Meta<RestrictionRef<'a>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.min
			.take()
			.map(|m| m.borrow().map(RestrictionRef::Min))
			.or_else(|| self.max.take().map(|m| m.borrow().map(RestrictionRef::Max)))
	}
}

impl<'a, M> DoubleEndedIterator for RestrictionsIter<'a, M> {
	fn next_back(&mut self) -> Option<Self::Item> {
		self.max
			.take()
			.map(|m| m.borrow().map(RestrictionRef::Max))
			.or_else(|| self.min.take().map(|m| m.borrow().map(RestrictionRef::Min)))
	}
}

pub enum Binding {
	Min(NonNegativeInteger),
	Max(NonNegativeInteger),
}

impl Binding {
	pub fn property(&self) -> Property {
		match self {
			Self::Min(_) => Property::MinCardinality,
			Self::Max(_) => Property::MaxCardinality,
		}
	}
}

#[derive(Debug)]
pub enum BindingRef<'a> {
	Min(&'a NonNegativeInteger),
	Max(&'a NonNegativeInteger),
}

impl<'a> BindingRef<'a> {
	pub fn property(&self) -> Property {
		match self {
			Self::Min(_) => Property::MinCardinality,
			Self::Max(_) => Property::MaxCardinality,
		}
	}
}
