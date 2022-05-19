#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Restrictions {
	min: u64,
	max: u64,
}

impl Default for Restrictions {
	fn default() -> Self {
		Self {
			min: 0,
			max: u64::MAX,
		}
	}
}

impl Restrictions {
	pub fn is_restricted(&self) -> bool {
		self.min != 0 || self.max != u64::MAX
	}

	pub fn min(&self) -> u64 {
		self.min
	}

	pub fn add_min(&mut self, min: u64) {
		self.min = core::cmp::max(self.min, min)
	}

	pub fn max(&self) -> u64 {
		self.max
	}

	pub fn add_max(&mut self, max: u64) {
		self.max = core::cmp::min(self.max, max)
	}

	pub fn iter(&self) -> Iter {
		Iter {
			min: if self.min != u64::MIN {
				Some(self.min)
			} else {
				None
			},
			max: if self.max != u64::MAX {
				Some(self.max)
			} else {
				None
			},
		}
	}
}

pub enum Restriction {
	MinInclusive(u64),
	MaxInclusive(u64),
}

pub struct Iter {
	min: Option<u64>,
	max: Option<u64>,
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
