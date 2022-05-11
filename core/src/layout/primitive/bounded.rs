use super::Primitive;
use btree_range_map::RangeSet;
use ordered_float::NotNan;
use std::ops::Bound;

/// Bounded primitive layout.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Bounded {
	Boolean,
	Integer(Integer),
	PositiveInteger(PositiveInteger),
	Float(Float),
	Double(Double),
	String(String),
	Time,
	Date,
	DateTime,
	Iri,
	Uri,
	Url,
}

impl Bounded {
	pub fn primitive(&self) -> Primitive {
		match self {
			Self::Boolean => Primitive::Boolean,
			Self::Integer(_) => Primitive::Integer,
			Self::PositiveInteger(_) => Primitive::PositiveInteger,
			Self::Float(_) => Primitive::Float,
			Self::Double(_) => Primitive::Double,
			Self::String(_) => Primitive::String,
			Self::Time => Primitive::Time,
			Self::Date => Primitive::Date,
			Self::DateTime => Primitive::DateTime,
			Self::Iri => Primitive::Iri,
			Self::Uri => Primitive::Uri,
			Self::Url => Primitive::Url,
		}
	}
}

impl From<Primitive> for Bounded {
	fn from(p: Primitive) -> Self {
		match p {
			Primitive::Boolean => Self::Boolean,
			Primitive::Integer => Self::Integer(Integer::default()),
			Primitive::PositiveInteger => Self::PositiveInteger(PositiveInteger::default()),
			Primitive::Float => Self::Float(Float::default()),
			Primitive::Double => Self::Double(Double::default()),
			Primitive::String => Self::String(String::default()),
			Primitive::Time => Self::Time,
			Primitive::Date => Self::Date,
			Primitive::DateTime => Self::DateTime,
			Primitive::Iri => Self::Iri,
			Primitive::Uri => Self::Uri,
			Primitive::Url => Self::Url,
		}
	}
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Integer {
	ranges: RangeSet<i64>,
}

impl Default for Integer {
	fn default() -> Self {
		let mut ranges = RangeSet::new();
		ranges.insert(i64::MIN..=i64::MAX);

		Self { ranges }
	}
}

impl Integer {
	pub fn bounded(&self) -> bool {
		if self.ranges.range_count() == 1 {
			let range = self.ranges.iter().next().unwrap();
			range.start != Bound::Included(i64::MIN) || range.end != Bound::Included(i64::MAX)
		} else {
			true
		}
	}

	pub fn ranges(&self) -> &RangeSet<i64> {
		&self.ranges
	}

	pub fn union(&mut self, other: Self) {
		self.ranges.extend(other.ranges)
	}
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct PositiveInteger {
	ranges: RangeSet<u64>,
}

impl Default for PositiveInteger {
	fn default() -> Self {
		let mut ranges = RangeSet::new();
		ranges.insert(u64::MIN..=u64::MAX);

		Self { ranges }
	}
}

impl PositiveInteger {
	pub fn bounded(&self) -> bool {
		if self.ranges.range_count() == 1 {
			let range = self.ranges.iter().next().unwrap();
			range.start != Bound::Included(u64::MIN) || range.end != Bound::Included(u64::MAX)
		} else {
			true
		}
	}

	pub fn ranges(&self) -> &RangeSet<u64> {
		&self.ranges
	}

	pub fn union(&mut self, other: Self) {
		self.ranges.extend(other.ranges)
	}
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Float {
	ranges: RangeSet<NotNan<f32>>,
}

impl Default for Float {
	fn default() -> Self {
		let mut ranges = RangeSet::new();
		ranges.insert(
			unsafe { NotNan::new_unchecked(f32::NEG_INFINITY) }..=unsafe {
				NotNan::new_unchecked(f32::INFINITY)
			},
		);

		Self { ranges }
	}
}

impl Float {
	pub fn bounded(&self) -> bool {
		if self.ranges.range_count() == 1 {
			let range = self.ranges.iter().next().unwrap();
			range.start != Bound::Included(unsafe { NotNan::new_unchecked(f32::NEG_INFINITY) })
				|| range.end != Bound::Included(unsafe { NotNan::new_unchecked(f32::INFINITY) })
		} else {
			true
		}
	}

	pub fn ranges(&self) -> &RangeSet<NotNan<f32>> {
		&self.ranges
	}

	pub fn union(&mut self, other: Self) {
		self.ranges.extend(other.ranges)
	}
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Double {
	ranges: RangeSet<NotNan<f64>>,
}

impl Default for Double {
	fn default() -> Self {
		let mut ranges = RangeSet::new();
		ranges.insert(
			unsafe { NotNan::new_unchecked(f64::NEG_INFINITY) }..=unsafe {
				NotNan::new_unchecked(f64::INFINITY)
			},
		);

		Self { ranges }
	}
}

impl Double {
	pub fn bounded(&self) -> bool {
		if self.ranges.range_count() == 1 {
			let range = self.ranges.iter().next().unwrap();
			range.start != Bound::Included(unsafe { NotNan::new_unchecked(f64::NEG_INFINITY) })
				|| range.end != Bound::Included(unsafe { NotNan::new_unchecked(f64::INFINITY) })
		} else {
			true
		}
	}

	pub fn ranges(&self) -> &RangeSet<NotNan<f64>> {
		&self.ranges
	}

	pub fn union(&mut self, other: Self) {
		self.ranges.extend(other.ranges)
	}
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct String {
	len_ranges: RangeSet<u64>,
}

impl Default for String {
	fn default() -> Self {
		let mut len_ranges = RangeSet::new();
		len_ranges.insert(u64::MIN..=u64::MAX);

		Self { len_ranges }
	}
}

impl String {
	pub fn bounded(&self) -> bool {
		if self.len_ranges.range_count() == 1 {
			let range = self.len_ranges.iter().next().unwrap();
			range.start != Bound::Included(u64::MIN) || range.end != Bound::Included(u64::MAX)
		} else {
			true
		}
	}

	pub fn len_ranges(&self) -> &RangeSet<u64> {
		&self.len_ranges
	}

	pub fn union(&mut self, other: Self) {
		self.len_ranges.extend(other.len_ranges)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn double_unbounded() {
		let mut bounds = Double::default();

		bounds
			.ranges
			.remove(NotNan::new(0.0).unwrap()..NotNan::new(1.0).unwrap());
		bounds
			.ranges
			.insert(NotNan::new(0.0).unwrap()..NotNan::new(1.0).unwrap());

		assert!(!bounds.bounded())
	}
}
