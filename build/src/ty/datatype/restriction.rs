use crate::{
	context::MapIds, functional_property_value, resource::BindingValueRef, Error,
	FunctionalPropertyValue, MetaValueExt,
};
use locspan::Meta;
use std::{cmp::Ordering, collections::BTreeMap};
use treeldr::{
	metadata::Merge,
	prop::UnknownProperty,
	ty::data::{restriction, RegExp},
	value, Id, TId, Value,
};

pub use treeldr::ty::data::restriction::Property;

#[derive(Clone)]
pub struct Definition<M> {
	restriction: FunctionalPropertyValue<Restriction, M>,
}

impl<M> Default for Definition<M> {
	fn default() -> Self {
		Self {
			restriction: FunctionalPropertyValue::default(),
		}
	}
}

impl<M> Definition<M> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn restriction(&self) -> &FunctionalPropertyValue<Restriction, M> {
		&self.restriction
	}

	pub fn restriction_mut(&mut self) -> &mut FunctionalPropertyValue<Restriction, M> {
		&mut self.restriction
	}

	pub fn bindings(&self) -> Bindings<M> {
		ClassBindings {
			restriction: self.restriction.iter(),
		}
	}

	pub fn set(
		&mut self,
		prop_cmp: impl Fn(TId<UnknownProperty>, TId<UnknownProperty>) -> Option<Ordering>,
		prop: Property,
		value: Meta<Value, M>,
	) -> Result<(), Error<M>>
	where
		M: Merge,
	{
		match prop {
			Property::MaxExclusive(p) => self.restriction.insert(
				p,
				prop_cmp,
				value
					.into_expected_numeric()?
					.map(|n| Restriction::Numeric(Numeric::MaxExclusive(n))),
			),
			Property::MaxInclusive(p) => self.restriction.insert(
				p,
				prop_cmp,
				value
					.into_expected_numeric()?
					.map(|n| Restriction::Numeric(Numeric::MaxInclusive(n))),
			),
			Property::MaxLength(p) => self.restriction.insert(
				p,
				prop_cmp,
				value
					.into_expected_integer()?
					.map(|n| Restriction::String(String::MaxLength(n))),
			),
			Property::MinExclusive(p) => self.restriction.insert(
				p,
				prop_cmp,
				value
					.into_expected_numeric()?
					.map(|n| Restriction::Numeric(Numeric::MinExclusive(n))),
			),
			Property::MinInclusive(p) => self.restriction.insert(
				p,
				prop_cmp,
				value
					.into_expected_numeric()?
					.map(|n| Restriction::Numeric(Numeric::MinInclusive(n))),
			),
			Property::MinLength(p) => self.restriction.insert(
				p,
				prop_cmp,
				value
					.into_expected_integer()?
					.map(|n| Restriction::String(String::MinLength(n))),
			),
			Property::Pattern(p) => self.restriction.insert(
				p,
				prop_cmp,
				value
					.into_expected_regexp()?
					.map(|p| Restriction::String(String::Pattern(p))),
			),
		}

		Ok(())
	}

	pub fn build(&self) -> Result<Meta<Restriction, M>, Error<M>>
	where
		M: Clone + Merge,
	{
		self.restriction
			.clone()
			.try_unwrap()
			.map_err(|_| todo!())?
			.into_required()
			.map(treeldr::RequiredFunctionalPropertyValue::into_meta_value)
			.ok_or_else(|| todo!())
	}
}

impl<M: Merge> MapIds for Definition<M> {
	fn map_ids(&mut self, _f: impl Fn(Id, Option<crate::Property>) -> Id) {
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
			Self::MinInclusive(v) => NumericBindingRef::MinInclusive(None, v),
			Self::MinExclusive(v) => NumericBindingRef::MinExclusive(None, v),
			Self::MaxInclusive(v) => NumericBindingRef::MaxInclusive(None, v),
			Self::MaxExclusive(v) => NumericBindingRef::MaxExclusive(None, v),
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
			Self::MinLength(v) => StringBindingRef::MinLength(None, v),
			Self::MaxLength(v) => StringBindingRef::MaxLength(None, v),
			Self::Pattern(v) => StringBindingRef::Pattern(None, v),
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

	pub fn build_real(
		self,
		_id: Id,
		meta: M,
	) -> Result<Meta<restriction::real::Restrictions, M>, Error<M>> {
		use restriction::real::{Max, Min};
		let mut r = restriction::real::Restrictions::default();

		for restriction in self.map.into_keys() {
			match restriction {
				Restriction::Numeric(restriction) => match restriction {
					Numeric::MinInclusive(value::Numeric::Real(min)) => {
						r.add_min(Min::Included(min))
					}
					Numeric::MinExclusive(value::Numeric::Real(min)) => {
						r.add_min(Min::Excluded(min))
					}
					Numeric::MaxInclusive(value::Numeric::Real(max)) => {
						r.add_max(Max::Included(max))
					}
					Numeric::MaxExclusive(value::Numeric::Real(max)) => {
						r.add_max(Max::Excluded(max))
					}
					_ => todo!(),
				},
				_ => {
					todo!()
				}
			}
		}

		Ok(Meta(r, meta))
	}

	pub fn build_float(
		self,
		_id: Id,
		meta: M,
	) -> Result<Meta<restriction::float::Restrictions, M>, Error<M>> {
		use restriction::float::{Max, Min};
		let mut r = restriction::float::Restrictions::default();

		for restriction in self.map.into_keys() {
			match restriction {
				Restriction::Numeric(restriction) => match restriction {
					Numeric::MinInclusive(value::Numeric::Float(min)) => {
						r.add_min(Min::Included(min))
					}
					Numeric::MinExclusive(value::Numeric::Float(min)) => {
						r.add_min(Min::Excluded(min))
					}
					Numeric::MaxInclusive(value::Numeric::Float(max)) => {
						r.add_max(Max::Included(max))
					}
					Numeric::MaxExclusive(value::Numeric::Float(max)) => {
						r.add_max(Max::Excluded(max))
					}
					_ => todo!(),
				},
				_ => {
					todo!()
				}
			}
		}

		Ok(Meta(r, meta))
	}

	pub fn build_double(
		self,
		_id: Id,
		meta: M,
	) -> Result<Meta<restriction::double::Restrictions, M>, Error<M>> {
		use restriction::double::{Max, Min};
		let mut r = restriction::double::Restrictions::default();

		for restriction in self.map.into_keys() {
			match restriction {
				Restriction::Numeric(restriction) => match restriction {
					Numeric::MinInclusive(value::Numeric::Double(min)) => {
						r.add_min(Min::Included(min))
					}
					Numeric::MinExclusive(value::Numeric::Double(min)) => {
						r.add_min(Min::Excluded(min))
					}
					Numeric::MaxInclusive(value::Numeric::Double(max)) => {
						r.add_max(Max::Included(max))
					}
					Numeric::MaxExclusive(value::Numeric::Double(max)) => {
						r.add_max(Max::Excluded(max))
					}
					_ => todo!(),
				},
				_ => {
					todo!()
				}
			}
		}

		Ok(Meta(r, meta))
	}

	pub fn build_string(
		self,
		_id: Id,
		meta: M,
	) -> Result<Meta<restriction::string::Restrictions, M>, Error<M>> {
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

		Ok(Meta(r, meta))
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

#[derive(Debug)]
pub enum NumericBindingRef<'a> {
	MinInclusive(Option<TId<UnknownProperty>>, &'a value::Numeric),
	MinExclusive(Option<TId<UnknownProperty>>, &'a value::Numeric),
	MaxInclusive(Option<TId<UnknownProperty>>, &'a value::Numeric),
	MaxExclusive(Option<TId<UnknownProperty>>, &'a value::Numeric),
}

impl<'a> NumericBindingRef<'a> {
	pub fn property(&self) -> Property {
		match self {
			Self::MinInclusive(p, _) => Property::MinInclusive(*p),
			Self::MinExclusive(p, _) => Property::MinExclusive(*p),
			Self::MaxInclusive(p, _) => Property::MaxInclusive(*p),
			Self::MaxExclusive(p, _) => Property::MaxExclusive(*p),
		}
	}

	pub fn value(&self) -> BindingValueRef<'a> {
		match self {
			Self::MinInclusive(_, v) => BindingValueRef::Numeric(v),
			Self::MinExclusive(_, v) => BindingValueRef::Numeric(v),
			Self::MaxInclusive(_, v) => BindingValueRef::Numeric(v),
			Self::MaxExclusive(_, v) => BindingValueRef::Numeric(v),
		}
	}
}

#[derive(Debug)]
pub enum StringBindingRef<'a> {
	MinLength(Option<TId<UnknownProperty>>, &'a value::Integer),
	MaxLength(Option<TId<UnknownProperty>>, &'a value::Integer),
	Pattern(Option<TId<UnknownProperty>>, &'a RegExp),
}

impl<'a> StringBindingRef<'a> {
	pub fn property(&self) -> Property {
		match self {
			Self::MinLength(p, _) => Property::MinLength(*p),
			Self::MaxLength(p, _) => Property::MaxLength(*p),
			Self::Pattern(p, _) => Property::Pattern(*p),
		}
	}

	pub fn value(&self) -> BindingValueRef<'a> {
		match self {
			Self::MinLength(_, v) => BindingValueRef::Integer(v),
			Self::MaxLength(_, v) => BindingValueRef::Integer(v),
			Self::Pattern(_, v) => BindingValueRef::RegExp(v),
		}
	}
}

#[derive(Debug)]
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

	pub fn value(&self) -> BindingValueRef<'a> {
		match self {
			Self::Numeric(b) => b.value(),
			Self::String(b) => b.value(),
		}
	}
}

pub struct ClassBindings<'a, M> {
	restriction: functional_property_value::Iter<'a, Restriction, M>,
}

pub type Bindings<'a, M> = ClassBindings<'a, M>;

impl<'a, M> Iterator for ClassBindings<'a, M> {
	type Item = Meta<ClassBindingRef<'a>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.restriction
			.next()
			.map(|m| m.value.map(Restriction::as_binding))
	}
}
