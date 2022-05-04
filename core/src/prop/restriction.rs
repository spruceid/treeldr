use crate::{ty, Ref};
use derivative::Derivative;
use std::collections::HashSet;

#[derive(Clone, Copy)]
pub struct Contradiction;

#[derive(Derivative)]
#[derivative(Default(bound = ""), Clone(bound = ""))]
pub struct Restrictions<F> {
	range: RangeRestrictions<F>,
	cardinality: CardinalityRestrictions,
}

impl<F> Restrictions<F> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn len(&self) -> usize {
		self.range.len() + self.cardinality.len()
	}

	pub fn is_empty(&self) -> bool {
		self.range.is_empty() && self.cardinality.is_empty()
	}

	pub fn iter(&self) -> Iter<F> {
		Iter {
			range: self.range.iter(),
			cardinality: self.cardinality.iter(),
		}
	}

	pub fn restrict(&mut self, restriction: Restriction<F>) -> Result<(), Contradiction> {
		match restriction {
			Restriction::Range(r) => {
				self.range.restrict(r);
				Ok(())
			}
			Restriction::Cardinality(c) => self.cardinality.restrict(c),
		}
	}

	pub fn clear(&mut self) {
		self.range.clear();
		self.cardinality.clear()
	}

	pub fn union_with(&self, other: &Self) -> Self {
		Self {
			range: self.range.union_with(&other.range),
			cardinality: self.cardinality.union_with(&other.cardinality),
		}
	}

	pub fn intersection_with(&self, other: &Self) -> Result<Self, Contradiction> {
		Ok(Self {
			range: self.range.intersection_with(&other.range),
			cardinality: self.cardinality.intersection_with(&other.cardinality)?,
		})
	}
}

pub struct Iter<'a, F> {
	range: RangeRestrictionsIter<'a, F>,
	cardinality: CardinalityRestrictionsIter,
}

impl<'a, F> Iterator for Iter<'a, F> {
	type Item = Restriction<F>;

	fn next(&mut self) -> Option<Self::Item> {
		self.range
			.next()
			.map(Restriction::Range)
			.or_else(|| self.cardinality.next().map(Restriction::Cardinality))
	}
}

impl<'a, F> IntoIterator for &'a Restrictions<F> {
	type Item = Restriction<F>;
	type IntoIter = Iter<'a, F>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

#[derive(Derivative)]
#[derivative(Default(bound = ""), Clone(bound = ""))]
pub struct RangeRestrictions<F> {
	all: HashSet<Ref<ty::Definition<F>>>,
	any: HashSet<Ref<ty::Definition<F>>>,
}

impl<F> RangeRestrictions<F> {
	pub fn len(&self) -> usize {
		self.all.len() + self.any.len()
	}

	pub fn is_empty(&self) -> bool {
		self.all.is_empty() && self.any.is_empty()
	}

	pub fn iter(&self) -> RangeRestrictionsIter<F> {
		RangeRestrictionsIter {
			all: self.all.iter().cloned(),
			any: self.any.iter().cloned(),
		}
	}

	pub fn restrict(&mut self, restriction: Range<F>) {
		match restriction {
			Range::All(r) => {
				self.all.insert(r);
			}
			Range::Any(r) => {
				self.any.insert(r);
			}
		}
	}

	pub fn clear(&mut self) {
		self.all.clear();
		self.any.clear();
	}

	pub fn union_with(&self, other: &Self) -> Self {
		Self {
			all: self.all.intersection(&other.all).cloned().collect(),
			any: self.any.intersection(&other.any).cloned().collect(),
		}
	}

	pub fn intersection_with(&self, other: &Self) -> Self {
		Self {
			all: self.all.union(&other.all).cloned().collect(),
			any: self.any.union(&other.any).cloned().collect(),
		}
	}
}

pub struct RangeRestrictionsIter<'a, F> {
	all: std::iter::Cloned<std::collections::hash_set::Iter<'a, Ref<ty::Definition<F>>>>,
	any: std::iter::Cloned<std::collections::hash_set::Iter<'a, Ref<ty::Definition<F>>>>,
}

impl<'a, F> Iterator for RangeRestrictionsIter<'a, F> {
	type Item = Range<F>;

	fn next(&mut self) -> Option<Self::Item> {
		match self.any.next() {
			Some(ty_ref) => Some(Range::Any(ty_ref)),
			None => self.all.next().map(Range::All),
		}
	}
}

#[derive(Default, Clone)]
pub struct CardinalityRestrictions {
	min: Option<u32>,
	max: Option<u32>,
}

impl CardinalityRestrictions {
	pub fn len(&self) -> usize {
		match (self.min, self.max) {
			(Some(min), Some(max)) if min == max => 1,
			(Some(_), Some(_)) => 2,
			(Some(_), None) => 1,
			(None, Some(_)) => 1,
			(None, None) => 0,
		}
	}

	pub fn is_empty(&self) -> bool {
		self.min.is_none() && self.max.is_none()
	}

	pub fn iter(&self) -> CardinalityRestrictionsIter {
		CardinalityRestrictionsIter {
			min: self.min,
			max: self.max,
		}
	}

	pub fn restrict(&mut self, restriction: Cardinality) -> Result<(), Contradiction> {
		match restriction {
			Cardinality::AtLeast(min) => {
				if let Some(max) = self.max {
					if min > max {
						return Err(Contradiction);
					}
				}

				self.min = Some(min)
			}
			Cardinality::AtMost(max) => {
				if let Some(min) = self.min {
					if min > max {
						return Err(Contradiction);
					}
				}

				self.max = Some(max)
			}
			Cardinality::Exactly(n) => {
				if let Some(min) = self.min {
					if min > n {
						return Err(Contradiction);
					}
				}

				if let Some(max) = self.max {
					if n > max {
						return Err(Contradiction);
					}
				}

				self.min = Some(n);
				self.max = Some(n);
			}
		}

		Ok(())
	}

	pub fn clear(&mut self) {
		self.min = None;
		self.max = None;
	}

	pub fn union_with(&self, other: &Self) -> Self {
		let min = match (self.min, other.min) {
			(Some(a), Some(b)) => Some(std::cmp::min(a, b)),
			_ => None,
		};

		let max = match (self.max, other.max) {
			(Some(a), Some(b)) => Some(std::cmp::max(a, b)),
			_ => None,
		};

		Self { min, max }
	}

	pub fn intersection_with(&self, other: &Self) -> Result<Self, Contradiction> {
		let min = match (self.min, other.min) {
			(Some(a), Some(b)) => Some(std::cmp::max(a, b)),
			(Some(min), None) => Some(min),
			(None, Some(min)) => Some(min),
			(None, None) => None,
		};

		let max = match (self.max, other.max) {
			(Some(a), Some(b)) => Some(std::cmp::min(a, b)),
			(Some(max), None) => Some(max),
			(None, Some(max)) => Some(max),
			_ => None,
		};

		if let (Some(min), Some(max)) = (min, max) {
			if min > max {
				return Err(Contradiction);
			}
		}

		Ok(Self { min, max })
	}
}

pub struct CardinalityRestrictionsIter {
	min: Option<u32>,
	max: Option<u32>,
}

impl Iterator for CardinalityRestrictionsIter {
	type Item = Cardinality;

	fn next(&mut self) -> Option<Self::Item> {
		if self.min == self.max {
			self.min.take();
			self.max.take().map(Cardinality::Exactly)
		} else {
			match self.min.take() {
				Some(min) => Some(Cardinality::AtLeast(min)),
				None => self.max.take().map(Cardinality::AtMost),
			}
		}
	}
}

/// Property restriction.
#[derive(Derivative)]
#[derivative(
	Clone(bound = ""),
	Copy(bound = ""),
	PartialEq(bound = ""),
	Eq(bound = "")
)]
pub enum Restriction<F> {
	/// Range restriction.
	Range(Range<F>),

	/// Cardinality restriction.
	Cardinality(Cardinality),
}

/// Property range restriction.
#[derive(Derivative)]
#[derivative(
	Clone(bound = ""),
	Copy(bound = ""),
	PartialEq(bound = ""),
	Eq(bound = "")
)]
pub enum Range<F> {
	/// At least one value must be an instance of the given type.
	Any(Ref<ty::Definition<F>>),

	/// All the values must be instances of the given type.
	All(Ref<ty::Definition<F>>),
}

/// Property cardinality restriction.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Cardinality {
	/// The property must have at least the given number of values.
	AtLeast(u32),

	/// The property must have at most the given number of values.
	AtMost(u32),

	/// The property must have exactly the given number of values.
	Exactly(u32),
}
