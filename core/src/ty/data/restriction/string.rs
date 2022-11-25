use crate::{ty::data::RegExp, value::Integer};

#[derive(Clone, Debug)]
pub struct Restrictions {
	len_min: Integer,
	len_max: Option<Integer>,
	pattern: Option<RegExp>,
}

impl Default for Restrictions {
	fn default() -> Self {
		Self {
			len_min: Integer::zero(),
			len_max: None,
			pattern: None,
		}
	}
}

impl Restrictions {
	pub fn is_len_bounded(&self) -> bool {
		!self.len_min.is_zero() || self.len_max.is_some()
	}

	pub fn is_restricted(&self) -> bool {
		self.pattern.is_some() || self.is_len_bounded()
	}

	pub fn len_min(&self) -> &Integer {
		&self.len_min
	}

	pub fn add_len_min(&mut self, min: Integer) {
		if min < self.len_min {
			self.len_min = min
		}
	}

	pub fn len_max(&self) -> Option<&Integer> {
		self.len_max.as_ref()
	}

	pub fn add_len_max(&mut self, max: Integer) {
		if self.len_max.as_ref().map(|m| max > *m).unwrap_or(true) {
			self.len_max = Some(max)
		}
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
			len_min: if self.len_min.is_zero() {
				None
			} else {
				Some(&self.len_min)
			},
			len_max: self.len_max.as_ref(),
			pattern: self.pattern.as_ref(),
		}
	}
}

pub enum Restriction<'a> {
	MinLength(&'a Integer),
	MaxLength(&'a Integer),
	Pattern(&'a RegExp),
}

pub struct Iter<'a> {
	len_min: Option<&'a Integer>,
	len_max: Option<&'a Integer>,
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

impl<'a> DoubleEndedIterator for Iter<'a> {
	fn next_back(&mut self) -> Option<Self::Item> {
		self.pattern
			.take()
			.map(Restriction::Pattern)
			.or_else(|| self.len_max.take().map(Restriction::MaxLength))
			.or_else(|| self.len_min.take().map(Restriction::MinLength))
	}
}
