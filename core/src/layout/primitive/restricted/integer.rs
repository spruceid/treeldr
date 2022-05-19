#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Restrictions {
	min: i64,
	max: i64,
}

impl Default for Restrictions {
	fn default() -> Self {
		Self {
			min: i64::MIN,
			max: i64::MAX,
		}
	}
}

impl Restrictions {
	pub fn is_restricted(&self) -> bool {
		self.min != i64::MIN || self.max != i64::MAX
	}

	pub fn min(&self) -> i64 {
		self.min
	}

	pub fn max(&self) -> i64 {
		self.max
	}

	pub fn iter(&self) -> Iter {
		Iter {
			min: if self.min != i64::MIN {
				Some(self.min)
			} else {
				None
			},
			max: if self.max != i64::MAX {
				Some(self.max)
			} else {
				None
			},
		}
	}
}

pub enum Restriction {
	MinInclusive(i64),
	MaxInclusive(i64),
}

pub struct Iter {
	min: Option<i64>,
	max: Option<i64>,
}

impl Iterator for Iter {
	type Item = Restriction;

	fn next(&mut self) -> Option<Self::Item> {
		self.min
			.take()
			.map(Restriction::MinInclusive)
			.or_else(|| self.max.take().map(Restriction::MaxInclusive))
	}
}
