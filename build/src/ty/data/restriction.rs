use crate::Error;
use std::collections::BTreeMap;
use treeldr::{
	ty::data::{restriction, RegExp},
	value, Metadata, Id,
};

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Restriction {
	Real(Numeric<value::Real>),
	Rational(Numeric<value::Rational>),
	Integer(Numeric<value::Integer>),
	Float(Numeric<value::Float>),
	Double(Numeric<value::Double>),
	String(String),
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Numeric<T> {
	MinInclusive(T),
	MinExclusive(T),
	MaxInclusive(T),
	MaxExclusive(T),
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum String {
	MinLength(value::Integer),
	MaxLength(value::Integer),
	Pattern(RegExp),
}

#[derive(Clone)]
pub struct Restrictions<F> {
	map: BTreeMap<Restriction, Metadata<F>>,
}

impl<F> Default for Restrictions<F> {
	fn default() -> Self {
		Self {
			map: BTreeMap::new(),
		}
	}
}

impl<F> Restrictions<F> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn insert(&mut self, restriction: Restriction, causes: impl Into<Metadata<F>>)
	where
		F: Ord,
	{
		self.map
			.entry(restriction)
			.or_insert_with(Metadata::new)
			.extend(causes.into())
	}

	pub fn build_boolean(self, _id: Id) -> Result<(), Error<F>> {
		if let Some(_restriction) = self.map.into_iter().next() {
			todo!()
		}

		Ok(())
	}

	pub fn build_real(self, _id: Id) -> Result<restriction::real::Restrictions, Error<F>> {
		use restriction::real::{Max, Min};
		let mut r = restriction::real::Restrictions::default();

		for restriction in self.map.into_keys() {
			match restriction {
				Restriction::Real(restriction) => match restriction {
					Numeric::MinInclusive(min) => r.add_min(Min::Included(min)),
					Numeric::MinExclusive(min) => r.add_min(Min::Excluded(min)),
					Numeric::MaxInclusive(max) => r.add_max(Max::Included(max)),
					Numeric::MaxExclusive(max) => r.add_max(Max::Excluded(max)),
				},
				_ => {
					todo!()
				}
			}
		}

		Ok(r)
	}

	pub fn build_float(self, _id: Id) -> Result<restriction::float::Restrictions, Error<F>> {
		use restriction::float::{Max, Min};
		let mut r = restriction::float::Restrictions::default();

		for restriction in self.map.into_keys() {
			match restriction {
				Restriction::Float(restriction) => match restriction {
					Numeric::MinInclusive(min) => r.add_min(Min::Included(min)),
					Numeric::MinExclusive(min) => r.add_min(Min::Excluded(min)),
					Numeric::MaxInclusive(max) => r.add_max(Max::Included(max)),
					Numeric::MaxExclusive(max) => r.add_max(Max::Excluded(max)),
				},
				_ => {
					todo!()
				}
			}
		}

		Ok(r)
	}

	pub fn build_double(self, _id: Id) -> Result<restriction::double::Restrictions, Error<F>> {
		use restriction::double::{Max, Min};
		let mut r = restriction::double::Restrictions::default();

		for restriction in self.map.into_keys() {
			match restriction {
				Restriction::Double(restriction) => match restriction {
					Numeric::MinInclusive(min) => r.add_min(Min::Included(min)),
					Numeric::MinExclusive(min) => r.add_min(Min::Excluded(min)),
					Numeric::MaxInclusive(max) => r.add_max(Max::Included(max)),
					Numeric::MaxExclusive(max) => r.add_max(Max::Excluded(max)),
				},
				_ => {
					todo!()
				}
			}
		}

		Ok(r)
	}

	pub fn build_string(self, _id: Id) -> Result<restriction::string::Restrictions, Error<F>> {
		let mut r = restriction::string::Restrictions::default();

		for restriction in self.map.into_keys() {
			match restriction {
				Restriction::String(restriction) => match restriction {
					String::MinLength(min) => r.add_len_min(min),
					String::MaxLength(max) => r.add_len_max(max),
					String::Pattern(regexp) => r.add_pattern(regexp),
				},
				_ => {
					todo!()
				}
			}
		}

		Ok(r)
	}

	pub fn build_date(self, _id: Id) -> Result<(), Error<F>> {
		if let Some(_restriction) = self.map.into_iter().next() {
			todo!()
		}

		Ok(())
	}

	pub fn build_time(self, _id: Id) -> Result<(), Error<F>> {
		if let Some(_restriction) = self.map.into_iter().next() {
			todo!()
		}

		Ok(())
	}

	pub fn build_datetime(self, _id: Id) -> Result<(), Error<F>> {
		if let Some(_restriction) = self.map.into_iter().next() {
			todo!()
		}

		Ok(())
	}

	pub fn build_duration(self, _id: Id) -> Result<(), Error<F>> {
		if let Some(_restriction) = self.map.into_iter().next() {
			todo!()
		}

		Ok(())
	}
}
