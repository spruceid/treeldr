use crate::{Error, Single, single};
use std::collections::BTreeMap;
use locspan::Meta;
use treeldr::{
	metadata::Merge,
	ty::data::{restriction, RegExp},
	value, Id,
};

#[derive(Clone)]
pub struct Definition<M> {
	restriction: Single<Restriction, M>
}

impl<M> Definition<M> {
	pub fn new() -> Self {
		Self { restriction: Single::default() }
	}

	pub fn build(&self) -> Result<Meta<Restriction, M>, Error<M>> where M: Clone {
		self.restriction.clone().try_unwrap().map_err(|_| todo!())?.ok_or_else(|| todo!())
	}
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Restriction {
	Real(Numeric<value::Real>),
	Rational(Numeric<value::Rational>),
	Integer(Numeric<value::Integer>),
	Float(Numeric<value::Float>),
	Double(Numeric<value::Double>),
	String(String),
}

impl Restriction {
	pub fn as_binding(&self) -> ClassBindingRef {
		match self {
			Self::Real(r) => ClassBindingRef::Real(r.as_binding()),
			Self::Rational(r) => ClassBindingRef::Rational(r.as_binding()),
			Self::Integer(r) => ClassBindingRef::Integer(r.as_binding()),
			Self::Float(r) => ClassBindingRef::Float(r.as_binding()),
			Self::Double(r) => ClassBindingRef::Double(r.as_binding()),
			Self::String(r) => ClassBindingRef::String(r.as_binding()),
		}
	}
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Numeric<T> {
	MinInclusive(T),
	MinExclusive(T),
	MaxInclusive(T),
	MaxExclusive(T),
}

impl<T> Numeric<T> {
	pub fn as_binding(&self) -> NumericBindingRef<T> {
		match self {
			Self::MinInclusive(v) => NumericBindingRef::MinInclusive(v),
			Self::MinExclusive(v) => NumericBindingRef::MinExclusive(v),
			Self::MaxInclusive(v) => NumericBindingRef::MaxInclusive(v),
			Self::MaxExclusive(v) => NumericBindingRef::MaxExclusive(v)
		}
	}
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum String {
	MinLength(value::Integer),
	MaxLength(value::Integer),
	Pattern(RegExp),
}

impl String {
	pub fn as_binding(&self) -> StringBindingRef {
		match self {
			Self::MinLength(v) => StringBindingRef::MinLength(v),
			Self::MaxLength(v) => StringBindingRef::MaxLength(v),
			Self::Pattern(v) => StringBindingRef::Pattern(v)
		}
	}
}

#[derive(Clone)]
pub struct Restrictions<M> {
	map: BTreeMap<Restriction, M>,
}

impl<M> Default for Restrictions<M> {
	fn default() -> Self {
		Self {
			map: BTreeMap::new(),
		}
	}
}

impl<M> Restrictions<M> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn insert(&mut self, restriction: Restriction, causes: M)
	where
		M: Merge,
	{
		use std::collections::btree_map::Entry;
		match self.map.entry(restriction) {
			Entry::Vacant(entry) => {
				entry.insert(causes);
			}
			Entry::Occupied(mut entry) => entry.get_mut().merge_with(causes),
		}
	}

	pub fn build_boolean(self, _id: Id) -> Result<(), Error<M>> {
		if let Some(_restriction) = self.map.into_iter().next() {
			todo!()
		}

		Ok(())
	}

	pub fn build_real(self, _id: Id) -> Result<restriction::real::Restrictions, Error<M>> {
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

	pub fn build_float(self, _id: Id) -> Result<restriction::float::Restrictions, Error<M>> {
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

	pub fn build_double(self, _id: Id) -> Result<restriction::double::Restrictions, Error<M>> {
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

	pub fn build_string(self, _id: Id) -> Result<restriction::string::Restrictions, Error<M>> {
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

	pub fn build_date(self, _id: Id) -> Result<(), Error<M>> {
		if let Some(_restriction) = self.map.into_iter().next() {
			todo!()
		}

		Ok(())
	}

	pub fn build_time(self, _id: Id) -> Result<(), Error<M>> {
		if let Some(_restriction) = self.map.into_iter().next() {
			todo!()
		}

		Ok(())
	}

	pub fn build_datetime(self, _id: Id) -> Result<(), Error<M>> {
		if let Some(_restriction) = self.map.into_iter().next() {
			todo!()
		}

		Ok(())
	}

	pub fn build_duration(self, _id: Id) -> Result<(), Error<M>> {
		if let Some(_restriction) = self.map.into_iter().next() {
			todo!()
		}

		Ok(())
	}
}

pub enum NumericBindingRef<'a, T> {
	MinInclusive(&'a T),
	MinExclusive(&'a T),
	MaxInclusive(&'a T),
	MaxExclusive(&'a T)
}

pub enum StringBindingRef<'a> {
	MinLength(&'a value::Integer),
	MaxLength(&'a value::Integer),
	Pattern(&'a RegExp),
}

pub enum ClassBindingRef<'a> {
	Real(NumericBindingRef<'a, value::Real>),
	Rational(NumericBindingRef<'a, value::Rational>),
	Integer(NumericBindingRef<'a, value::Integer>),
	Float(NumericBindingRef<'a, value::Float>),
	Double(NumericBindingRef<'a, value::Double>),
	String(StringBindingRef<'a>),
}

pub type BindingRef<'a> = ClassBindingRef<'a>;

pub struct ClassBindings<'a, M> {
	restriction: single::Iter<'a, Restriction, M>
}

pub type Bindings<'a, M> = ClassBindings<'a, M>;

impl<'a, M> Iterator for ClassBindings<'a, M> {
	type Item = Meta<ClassBindingRef<'a>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.restriction.next().map(|m| m.map(Restriction::as_binding))
	}
}