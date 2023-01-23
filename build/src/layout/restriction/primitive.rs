use crate::{error, resource::BindingValueRef, Error};
use locspan::{MapLocErr, Meta};
use std::collections::BTreeMap;
pub use treeldr::{
	layout::{primitive::restriction, primitive::RegExp, Primitive},
	value, Id, MetaOption,
};
use treeldr::{
	metadata::Merge,
	vocab::{Term, Xsd},
	IriIndex,
};

pub use treeldr::layout::restriction::Property;

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Restriction {
	Numeric(Numeric),
	String(String),
}

impl Restriction {
	pub fn as_binding(&self) -> BindingRef {
		match self {
			Self::Numeric(r) => BindingRef::Numeric(r.as_binding()),
			Self::String(r) => BindingRef::String(r.as_binding()),
		}
	}
}

/// Numeric restriction.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Numeric {
	InclusiveMinimum(value::Numeric),
	ExclusiveMinimum(value::Numeric),
	InclusiveMaximum(value::Numeric),
	ExclusiveMaximum(value::Numeric),
}

impl Numeric {
	pub fn as_binding(&self) -> NumericBindingRef {
		match self {
			Self::InclusiveMinimum(v) => NumericBindingRef::InclusiveMinimum(v),
			Self::ExclusiveMinimum(v) => NumericBindingRef::ExclusiveMinimum(v),
			Self::InclusiveMaximum(v) => NumericBindingRef::InclusiveMaximum(v),
			Self::ExclusiveMaximum(v) => NumericBindingRef::ExclusiveMaximum(v),
		}
	}
}

/// Numeric restriction reference.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum NumericRef<'a> {
	InclusiveMinimum(&'a value::Real),
	ExclusiveMinimum(&'a value::Real),
	InclusiveMaximum(&'a value::Real),
	ExclusiveMaximum(&'a value::Real),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum StringProperty {
	Pattern,
}

impl StringProperty {
	pub fn id(&self) -> Id {
		match self {
			Self::Pattern => Id::Iri(IriIndex::Iri(Term::Xsd(Xsd::Pattern))),
		}
	}
}

/// String restriction.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum String {
	Pattern(RegExp),
}

impl String {
	pub fn as_binding(&self) -> StringBindingRef {
		match self {
			Self::Pattern(v) => StringBindingRef::Pattern(v),
		}
	}
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum StringRef<'a> {
	Pattern(&'a RegExp),
}

#[derive(Clone, Debug)]
pub struct Restrictions<M> {
	map: BTreeMap<Restriction, M>,
}

impl<M> Default for Restrictions<M> {
	fn default() -> Self {
		Self {
			map: BTreeMap::default(),
		}
	}
}

impl<M> PartialEq for Restrictions<M> {
	fn eq(&self, other: &Self) -> bool {
		self.map.len() == other.map.len()
			&& self.map.keys().zip(other.map.keys()).all(|(a, b)| a == b)
	}
}

impl<M> Eq for Restrictions<M> {}

impl<M> Restrictions<M> {
	pub fn is_empty(&self) -> bool {
		self.map.is_empty()
	}

	pub fn is_included_in(&self, other: &Self) -> bool {
		self.map.keys().all(|r| other.map.contains_key(r))
	}

	#[allow(clippy::should_implement_trait)]
	pub fn into_iter(self) -> impl DoubleEndedIterator<Item = Meta<Restriction, M>> {
		self.map.into_iter().map(|(k, v)| Meta(k, v))
	}
}

impl<M: Merge> Restrictions<M> {
	pub fn insert(&mut self, Meta(restriction, metadata): Meta<Restriction, M>) {
		use std::collections::btree_map::Entry;
		match self.map.entry(restriction) {
			Entry::Vacant(entry) => {
				entry.insert(metadata);
			}
			Entry::Occupied(mut entry) => entry.get_mut().merge_with(metadata),
		}
	}
}

impl<M: Clone> Restrictions<M> {
	pub fn build_boolean(self, id: Id) -> Result<(), Error<M>> {
		match self.map.into_iter().next() {
			Some((restriction, causes)) => Err(Error::new(
				error::LayoutDatatypeRestrictionInvalid {
					id,
					primitive: Primitive::Boolean,
					restriction,
				}
				.into(),
				causes,
			)),
			None => Ok(()),
		}
	}

	pub fn build_integer(self, id: Id) -> Result<restriction::integer::Restrictions<M>, Error<M>>
	where
		M: Merge,
	{
		let mut r = restriction::integer::Restrictions::default();

		for (restriction, causes) in self.map {
			match restriction {
				Restriction::Numeric(restriction) => match restriction {
					Numeric::InclusiveMinimum(min) => match min.into_integer() {
						Ok(min) => r.insert_min(Meta(min, causes)).map_loc_err(
							error::Description::LayoutDatatypeRestrictionIntegerConflict,
						)?,
						Err(_) => todo!(),
					},
					Numeric::ExclusiveMinimum(min) => match min.into_integer() {
						Ok(min) => r.insert_min(Meta(min + 1, causes)).map_loc_err(
							error::Description::LayoutDatatypeRestrictionIntegerConflict,
						)?,
						Err(_) => todo!(),
					},
					Numeric::InclusiveMaximum(max) => match max.into_integer() {
						Ok(max) => r.insert_max(Meta(max, causes)).map_loc_err(
							error::Description::LayoutDatatypeRestrictionIntegerConflict,
						)?,
						Err(_) => todo!(),
					},
					Numeric::ExclusiveMaximum(max) => match max.into_integer() {
						Ok(max) => r.insert_max(Meta(max - 1, causes)).map_loc_err(
							error::Description::LayoutDatatypeRestrictionIntegerConflict,
						)?,
						Err(_) => todo!(),
					},
				},
				other => {
					return Err(Error::new(
						error::LayoutDatatypeRestrictionInvalid {
							id,
							primitive: Primitive::Integer,
							restriction: other,
						}
						.into(),
						causes,
					))
				}
			}
		}

		Ok(r)
	}

	pub fn build_unsigned_integer(
		self,
		id: Id,
	) -> Result<restriction::unsigned::Restrictions<M>, Error<M>>
	where
		M: Merge,
	{
		let mut r = restriction::unsigned::Restrictions::default();

		for (restriction, causes) in self.map {
			match restriction {
				Restriction::Numeric(restriction) => match restriction {
					Numeric::InclusiveMinimum(min) => match min.into_non_negative_integer() {
						Ok(min) => r.insert_min(Meta(min, causes)).map_loc_err(
							error::Description::LayoutDatatypeRestrictionUnsignedConflict,
						)?,
						Err(_) => todo!(),
					},
					Numeric::ExclusiveMinimum(min) => match min.into_non_negative_integer() {
						Ok(min) => r.insert_min(Meta(min + 1, causes)).map_loc_err(
							error::Description::LayoutDatatypeRestrictionUnsignedConflict,
						)?,
						Err(_) => todo!(),
					},
					Numeric::InclusiveMaximum(max) => match max.into_non_negative_integer() {
						Ok(max) => r.insert_max(Meta(max, causes)).map_loc_err(
							error::Description::LayoutDatatypeRestrictionUnsignedConflict,
						)?,
						Err(_) => todo!(),
					},
					Numeric::ExclusiveMaximum(max) => match max.into_non_negative_integer() {
						Ok(max) => r.insert_max(Meta(max - 1, causes)).map_loc_err(
							error::Description::LayoutDatatypeRestrictionUnsignedConflict,
						)?,
						Err(_) => todo!(),
					},
				},
				other => {
					return Err(Error::new(
						error::LayoutDatatypeRestrictionInvalid {
							id,
							primitive: Primitive::Integer,
							restriction: other,
						}
						.into(),
						causes,
					))
				}
			}
		}

		Ok(r)
	}

	pub fn build_float(self, id: Id) -> Result<restriction::float::Restrictions<M>, Error<M>>
	where
		M: Merge,
	{
		use restriction::float::{Max, Min};
		let mut r = restriction::float::Restrictions::default();

		for (restriction, causes) in self.map {
			match restriction {
				Restriction::Numeric(restriction) => match restriction {
					Numeric::InclusiveMinimum(min) => match min.into_float() {
						Ok(min) => r.insert_min(Meta(Min::Included(min), causes)).map_loc_err(
							error::Description::LayoutDatatypeRestrictionFloatConflict,
						)?,
						Err(_) => todo!(),
					},
					Numeric::ExclusiveMinimum(min) => match min.into_float() {
						Ok(min) => r.insert_min(Meta(Min::Excluded(min), causes)).map_loc_err(
							error::Description::LayoutDatatypeRestrictionFloatConflict,
						)?,
						Err(_) => todo!(),
					},
					Numeric::InclusiveMaximum(max) => match max.into_float() {
						Ok(max) => r.insert_max(Meta(Max::Included(max), causes)).map_loc_err(
							error::Description::LayoutDatatypeRestrictionFloatConflict,
						)?,
						Err(_) => todo!(),
					},
					Numeric::ExclusiveMaximum(max) => match max.into_float() {
						Ok(max) => r.insert_max(Meta(Max::Excluded(max), causes)).map_loc_err(
							error::Description::LayoutDatatypeRestrictionFloatConflict,
						)?,
						Err(_) => todo!(),
					},
				},
				other => {
					return Err(Error::new(
						error::LayoutDatatypeRestrictionInvalid {
							id,
							primitive: Primitive::Integer,
							restriction: other,
						}
						.into(),
						causes,
					))
				}
			}
		}

		Ok(r)
	}

	pub fn build_double(self, id: Id) -> Result<restriction::double::Restrictions<M>, Error<M>>
	where
		M: Merge,
	{
		use restriction::double::{Max, Min};
		let mut r = restriction::double::Restrictions::default();

		for (restriction, causes) in self.map {
			match restriction {
				Restriction::Numeric(restriction) => match restriction {
					Numeric::InclusiveMinimum(min) => match min.into_double() {
						Ok(min) => r.insert_min(Meta(Min::Included(min), causes)).map_loc_err(
							error::Description::LayoutDatatypeRestrictionDoubleConflict,
						)?,
						Err(_) => todo!(),
					},
					Numeric::ExclusiveMinimum(min) => match min.into_double() {
						Ok(min) => r.insert_min(Meta(Min::Excluded(min), causes)).map_loc_err(
							error::Description::LayoutDatatypeRestrictionDoubleConflict,
						)?,
						Err(_) => todo!(),
					},
					Numeric::InclusiveMaximum(max) => match max.into_double() {
						Ok(max) => r.insert_max(Meta(Max::Included(max), causes)).map_loc_err(
							error::Description::LayoutDatatypeRestrictionDoubleConflict,
						)?,
						Err(_) => todo!(),
					},
					Numeric::ExclusiveMaximum(max) => match max.into_double() {
						Ok(max) => r.insert_max(Meta(Max::Excluded(max), causes)).map_loc_err(
							error::Description::LayoutDatatypeRestrictionDoubleConflict,
						)?,
						Err(_) => todo!(),
					},
				},
				other => {
					return Err(Error::new(
						error::LayoutDatatypeRestrictionInvalid {
							id,
							primitive: Primitive::Integer,
							restriction: other,
						}
						.into(),
						causes,
					))
				}
			}
		}

		Ok(r)
	}

	pub fn build_string(self, id: Id) -> Result<restriction::string::Restrictions<M>, Error<M>>
	where
		M: Merge,
	{
		let mut p = restriction::string::Restrictions::default();

		for (restriction, causes) in self.map.into_iter() {
			match restriction {
				Restriction::String(restriction) => match restriction {
					String::Pattern(regexp) => p.insert_pattern(Meta(regexp, causes)),
				},
				other => {
					return Err(Error::new(
						error::LayoutDatatypeRestrictionInvalid {
							id,
							primitive: Primitive::String,
							restriction: other,
						}
						.into(),
						causes,
					))
				}
			}
		}

		Ok(p)
	}

	pub fn build_time(self, id: Id) -> Result<(), Error<M>> {
		match self.map.into_iter().next() {
			Some((restriction, causes)) => Err(Error::new(
				error::LayoutDatatypeRestrictionInvalid {
					id,
					primitive: Primitive::Time,
					restriction,
				}
				.into(),
				causes,
			)),
			None => Ok(()),
		}
	}

	pub fn build_date(self, id: Id) -> Result<(), Error<M>> {
		match self.map.into_iter().next() {
			Some((restriction, causes)) => Err(Error::new(
				error::LayoutDatatypeRestrictionInvalid {
					id,
					primitive: Primitive::Date,
					restriction,
				}
				.into(),
				causes,
			)),
			None => Ok(()),
		}
	}

	pub fn build_date_time(self, id: Id) -> Result<(), Error<M>> {
		match self.map.into_iter().next() {
			Some((restriction, causes)) => Err(Error::new(
				error::LayoutDatatypeRestrictionInvalid {
					id,
					primitive: Primitive::DateTime,
					restriction,
				}
				.into(),
				causes,
			)),
			None => Ok(()),
		}
	}

	pub fn build_iri(self, id: Id) -> Result<(), Error<M>> {
		match self.map.into_iter().next() {
			Some((restriction, causes)) => Err(Error::new(
				error::LayoutDatatypeRestrictionInvalid {
					id,
					primitive: Primitive::Iri,
					restriction,
				}
				.into(),
				causes,
			)),
			None => Ok(()),
		}
	}

	pub fn build_uri(self, id: Id) -> Result<(), Error<M>> {
		match self.map.into_iter().next() {
			Some((restriction, causes)) => Err(Error::new(
				error::LayoutDatatypeRestrictionInvalid {
					id,
					primitive: Primitive::Uri,
					restriction,
				}
				.into(),
				causes,
			)),
			None => Ok(()),
		}
	}

	pub fn build_url(self, id: Id) -> Result<(), Error<M>> {
		match self.map.into_iter().next() {
			Some((restriction, causes)) => Err(Error::new(
				error::LayoutDatatypeRestrictionInvalid {
					id,
					primitive: Primitive::Url,
					restriction,
				}
				.into(),
				causes,
			)),
			None => Ok(()),
		}
	}
}

pub enum BindingRef<'a> {
	Numeric(NumericBindingRef<'a>),
	String(StringBindingRef<'a>),
}

impl<'a> BindingRef<'a> {
	pub fn property(&self) -> Property {
		match self {
			Self::Numeric(b) => b.property(),
			Self::String(b) => b.property(),
		}
	}

	pub fn value<M>(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::Numeric(b) => b.value(),
			Self::String(b) => b.value(),
		}
	}
}

pub enum NumericBindingRef<'a> {
	InclusiveMinimum(&'a value::Numeric),
	ExclusiveMinimum(&'a value::Numeric),
	InclusiveMaximum(&'a value::Numeric),
	ExclusiveMaximum(&'a value::Numeric),
}

impl<'a> NumericBindingRef<'a> {
	pub fn property(&self) -> Property {
		match self {
			Self::InclusiveMinimum(_) => Property::InclusiveMinimum,
			Self::ExclusiveMinimum(_) => Property::ExclusiveMinimum,
			Self::InclusiveMaximum(_) => Property::InclusiveMaximum,
			Self::ExclusiveMaximum(_) => Property::ExclusiveMaximum,
		}
	}

	pub fn value<M>(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::InclusiveMinimum(v) => BindingValueRef::Numeric(v),
			Self::ExclusiveMinimum(v) => BindingValueRef::Numeric(v),
			Self::InclusiveMaximum(v) => BindingValueRef::Numeric(v),
			Self::ExclusiveMaximum(v) => BindingValueRef::Numeric(v),
		}
	}
}

pub enum StringBindingRef<'a> {
	Pattern(&'a RegExp),
}

impl<'a> StringBindingRef<'a> {
	pub fn property(&self) -> Property {
		match self {
			Self::Pattern(_) => Property::Pattern,
		}
	}

	pub fn value<M>(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::Pattern(v) => BindingValueRef::RegExp(v),
		}
	}
}
