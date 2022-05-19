use crate::ty::data::RegExp;

#[derive(Clone, Debug)]
pub struct Restrictions {
	len_min: u64,
	len_max: u64,
	pattern: Option<RegExp>,
}

impl Default for Restrictions {
	fn default() -> Self {
		Self {
			len_min: 0,
			len_max: u64::MAX,
			pattern: None,
		}
	}
}

impl Restrictions {
	pub fn is_len_bounded(&self) -> bool {
		self.len_min > 0 || self.len_max < u64::MAX
	}

	pub fn is_restricted(&self) -> bool {
		self.pattern.is_some() || self.is_len_bounded()
	}

	pub fn len_min(&self) -> u64 {
		self.len_min
	}

	pub fn len_max(&self) -> u64 {
		self.len_max
	}

	pub fn pattern(&self) -> Option<&RegExp> {
		self.pattern.as_ref()
	}

	pub fn add_pattern(&mut self, regexp: RegExp) {
		if self.pattern.is_some() && self.pattern.as_ref() != Some(&regexp) {
			todo!("intersect patterns")
		}

		self.pattern = Some(regexp)
	}

	pub fn is_simple_regexp(&self) -> bool {
		self.pattern.is_some() && !self.is_len_bounded()
	}

	pub fn as_simple_regexp(&self) -> Option<&RegExp> {
		if self.is_len_bounded() {
			None
		} else {
			self.pattern.as_ref()
		}
	}

	pub fn iter(&self) -> Iter {
		Iter {
			len_min: if self.len_min != u64::MIN {
				Some(self.len_min)
			} else {
				None
			},
			len_max: if self.len_max != u64::MAX {
				Some(self.len_max)
			} else {
				None
			},
			pattern: self.pattern.as_ref(),
		}
	}
}

pub enum Restriction<'a> {
	MinLength(u64),
	MaxLength(u64),
	Pattern(&'a RegExp),
}

pub struct Iter<'a> {
	len_min: Option<u64>,
	len_max: Option<u64>,
	pattern: Option<&'a RegExp>,
}

impl<'a> Iterator for Iter<'a> {
	type Item = Restriction<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		self.len_min
			.take()
			.map(Restriction::MinLength)
			.or_else(|| self.len_max.take().map(Restriction::MaxLength))
			.or_else(|| self.pattern.take().map(Restriction::Pattern))
	}
}
