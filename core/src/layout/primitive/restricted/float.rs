use crate::value::Float;
use std::cmp::{Ord, Ordering};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Min {
	Included(Float),
	Excluded(Float),
}

impl Min {
	pub fn value(&self) -> Float {
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
		*self != Self::Included(Float::NEG_INFINITY)
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

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Max {
	Included(Float),
	Excluded(Float),
}

impl Max {
	pub fn value(&self) -> Float {
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
		*self != Self::Included(Float::INFINITY)
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

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Restrictions {
	min: Min,
	max: Max,
}

impl Default for Restrictions {
	fn default() -> Self {
		Self {
			min: Min::Included(Float::NEG_INFINITY),
			max: Max::Included(Float::INFINITY),
		}
	}
}

impl Restrictions {
	pub fn is_restricted(&self) -> bool {
		self.min.is_bounded() || self.max.is_bounded()
	}

	pub fn min(&self) -> Min {
		self.min
	}

	pub fn max(&self) -> Max {
		self.max
	}

	pub fn iter(&self) -> Iter {
		Iter {
			min: if self.min.is_bounded() {
				Some(self.min)
			} else {
				None
			},
			max: if self.max.is_bounded() {
				Some(self.max)
			} else {
				None
			},
		}
	}
}

pub enum Restriction {
	Min(Min),
	Max(Max),
}

pub struct Iter {
	min: Option<Min>,
	max: Option<Max>,
}

impl Iterator for Iter {
	type Item = Restriction;

	fn size_hint(&self) -> (usize, Option<usize>) {
		let len = if self.min.is_some() { 1 } else { 0 } + if self.max.is_some() { 1 } else { 0 };
		(len, Some(len))
	}

	fn next(&mut self) -> Option<Self::Item> {
		self.min
			.take()
			.map(Restriction::Min)
			.or_else(|| self.max.take().map(Restriction::Max))
	}
}

impl ExactSizeIterator for Iter {}
