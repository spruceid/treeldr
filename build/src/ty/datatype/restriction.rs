use crate::{Error, Single, single, resource::BindingValueRef, context::MapIds};
use std::collections::BTreeMap;
use locspan::Meta;
use treeldr::{
	metadata::Merge,
	ty::data::{restriction, RegExp},
	value, Id,
};

pub use treeldr::ty::data::restriction::Property;

#[derive(Clone)]
pub struct Definition<M> {
	restriction: Single<Restriction, M>
}

impl<M> Definition<M> {
	pub fn new() -> Self {
		Self { restriction: Single::default() }
	}

	pub fn bindings(&self) -> Bindings<M> {
		ClassBindings { restriction: self.restriction.iter() }
	}

	pub fn build(&self) -> Result<Meta<Restriction, M>, Error<M>> where M: Clone {
		self.restriction.clone().try_unwrap().map_err(|_| todo!())?.ok_or_else(|| todo!())
	}
}

impl<M: Merge> MapIds for Definition<M> {
	fn map_ids(&mut self, _f: impl Fn(Id) -> Id) {
		// nothing.
	}
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Restriction {
	Numeric(Numeric),
	String(String),
}

impl Restriction {
	pub fn as_binding(&self) -> ClassBindingRef {
		match self {
			Self::Numeric(r) => ClassBindingRef::Numeric(r.as_binding()),
			Self::String(r) => ClassBindingRef::String(r.as_binding()),
		}
	}
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Numeric {
	MinInclusive(value::Numeric),
	MinExclusive(value::Numeric),
	MaxInclusive(value::Numeric),
	MaxExclusive(value::Numeric),
}

impl Numeric {
	pub fn as_binding(&self) -> NumericBindingRef {
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
				Restriction::Numeric(restriction) => match restriction {
					Numeric::MinInclusive(value::Numeric::Real(min)) => r.add_min(Min::Included(min)),
					Numeric::MinExclusive(value::Numeric::Real(min)) => r.add_min(Min::Excluded(min)),
					Numeric::MaxInclusive(value::Numeric::Real(max)) => r.add_max(Max::Included(max)),
					Numeric::MaxExclusive(value::Numeric::Real(max)) => r.add_max(Max::Excluded(max)),
					_ => todo!()
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
				Restriction::Numeric(restriction) => match restriction {
					Numeric::MinInclusive(value::Numeric::Float(min)) => r.add_min(Min::Included(min)),
					Numeric::MinExclusive(value::Numeric::Float(min)) => r.add_min(Min::Excluded(min)),
					Numeric::MaxInclusive(value::Numeric::Float(max)) => r.add_max(Max::Included(max)),
					Numeric::MaxExclusive(value::Numeric::Float(max)) => r.add_max(Max::Excluded(max)),
					_ => todo!()
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
				Restriction::Numeric(restriction) => match restriction {
					Numeric::MinInclusive(value::Numeric::Double(min)) => r.add_min(Min::Included(min)),
					Numeric::MinExclusive(value::Numeric::Double(min)) => r.add_min(Min::Excluded(min)),
					Numeric::MaxInclusive(value::Numeric::Double(max)) => r.add_max(Max::Included(max)),
					Numeric::MaxExclusive(value::Numeric::Double(max)) => r.add_max(Max::Excluded(max)),
					_ => todo!()
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

pub enum NumericBindingRef<'a> {
	MinInclusive(&'a value::Numeric),
	MinExclusive(&'a value::Numeric),
	MaxInclusive(&'a value::Numeric),
	MaxExclusive(&'a value::Numeric)
}

impl<'a> NumericBindingRef<'a> {
	pub fn property(&self) -> Property {
		match self {
			Self::MinInclusive(_) => Property::MinInclusive,
			Self::MinExclusive(_) => Property::MinExclusive,
			Self::MaxInclusive(_) => Property::MaxInclusive,
			Self::MaxExclusive(_) => Property::MaxExclusive
		}
	}

	pub fn value<M>(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::MinInclusive(v) => BindingValueRef::Numeric(v),
			Self::MinExclusive(v) => BindingValueRef::Numeric(v),
			Self::MaxInclusive(v) => BindingValueRef::Numeric(v),
			Self::MaxExclusive(v) => BindingValueRef::Numeric(v)
		}
	}
}

pub enum StringBindingRef<'a> {
	MinLength(&'a value::Integer),
	MaxLength(&'a value::Integer),
	Pattern(&'a RegExp),
}

impl<'a> StringBindingRef<'a> {
	pub fn property(&self) -> Property {
		match self {
			Self::MinLength(_) => Property::MinLength,
			Self::MaxLength(_) => Property::MaxLength,
			Self::Pattern(_) => Property::Pattern
		}
	}

	pub fn value<M>(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::MinLength(v) => BindingValueRef::Integer(v),
			Self::MaxLength(v) => BindingValueRef::Integer(v),
			Self::Pattern(v) => BindingValueRef::RegExp(v)
		}
	}
}

pub enum ClassBindingRef<'a> {
	Numeric(NumericBindingRef<'a>),
	String(StringBindingRef<'a>),
}

pub type BindingRef<'a> = ClassBindingRef<'a>;

impl<'a> ClassBindingRef<'a> {
	pub fn property(&self) -> Property {
		match self {
			Self::Numeric(b) => b.property(),
			Self::String(b) => b.property(),
		}
	}

	pub fn value<M>(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::Numeric(b) => b.value(),
			Self::String(b) => b.value()
		}
	}
}

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