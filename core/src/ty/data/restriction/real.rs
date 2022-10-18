use crate::value::Real;
use std::cmp::{Ord, Ordering};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Min {
	Included(Real),
	Excluded(Real),
}

impl Min {
	pub fn value(&self) -> &Real {
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
}

fn gt(a: &Min, b: Option<&Min>) -> bool {
	match (a, b) {
		(_, None) => true,
		(a, Some(b)) => a > b,
	}
}

impl Ord for Min {
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

impl PartialOrd for Min {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Max {
	Included(Real),
	Excluded(Real),
}

impl Max {
	pub fn value(&self) -> &Real {
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
}

fn lt(a: &Max, b: Option<&Max>) -> bool {
	match (a, b) {
		(_, None) => true,
		(a, Some(b)) => a < b,
	}
}

impl Ord for Max {
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

impl PartialOrd for Max {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

#[derive(Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Restrictions {
	min: Option<Min>,
	max: Option<Max>,
}

impl Restrictions {
	pub fn is_restricted(&self) -> bool {
		self.min.is_some() || self.max.is_some()
	}

	pub fn min(&self) -> Option<&Min> {
		self.min.as_ref()
	}

	pub fn add_min(&mut self, min: Min) {
		if gt(&min, self.min.as_ref()) {
			self.min = Some(min)
		}
	}

	pub fn max(&self) -> Option<&Max> {
		self.max.as_ref()
	}

	pub fn add_max(&mut self, max: Max) {
		if lt(&max, self.max.as_ref()) {
			self.max = Some(max)
		}
	}

	pub fn iter(&self) -> Iter {
		Iter {
			min: self.min.as_ref(),
			max: self.max.as_ref(),
		}
	}
}

pub enum Restriction<'a> {
	Min(&'a Min),
	Max(&'a Max),
}

pub struct Iter<'a> {
	min: Option<&'a Min>,
	max: Option<&'a Max>,
}

impl<'a> Iterator for Iter<'a> {
	type Item = Restriction<'a>;

	fn size_hint(&self) -> (usize, Option<usize>) {
		let len = usize::from(self.min.is_some()) + usize::from(self.max.is_some());
		(len, Some(len))
	}

	fn next(&mut self) -> Option<Self::Item> {
		self.min
			.take()
			.map(Restriction::Min)
			.or_else(|| self.max.take().map(Restriction::Max))
	}
}

impl<'a> ExactSizeIterator for Iter<'a> {}
