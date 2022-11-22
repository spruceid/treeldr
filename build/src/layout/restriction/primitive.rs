use crate::{error, Error};
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

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Property {
	Numeric(NumericProperty),
	String(StringProperty),
}

impl Property {
	pub fn id(&self) -> Id {
		match self {
			Self::Numeric(p) => p.id(),
			Self::String(p) => p.id(),
		}
	}
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Restriction {
	Numeric(Numeric),
	String(String),
}

impl Restriction {
	pub fn as_binding(&self) -> BindingRef {
		match self {
			Self::Numeric(r) => BindingRef::Numeric(r.as_binding()),
			Self::String(r) => BindingRef::String(r.as_binding())
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum NumericProperty {
	InclusiveMinimum,
	ExclusiveMinimum,
	InclusiveMaximum,
	ExclusiveMaximum,
}

impl NumericProperty {
	pub fn id(&self) -> Id {
		match self {
			Self::InclusiveMinimum => Id::Iri(IriIndex::Iri(Term::Xsd(Xsd::MinInclusive))),
			Self::ExclusiveMinimum => Id::Iri(IriIndex::Iri(Term::Xsd(Xsd::MinExclusive))),
			Self::InclusiveMaximum => Id::Iri(IriIndex::Iri(Term::Xsd(Xsd::MaxInclusive))),
			Self::ExclusiveMaximum => Id::Iri(IriIndex::Iri(Term::Xsd(Xsd::MaxExclusive))),
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
			Self::ExclusiveMaximum(v) => NumericBindingRef::ExclusiveMaximum(v)
		}
	}
}

/// Numeric restriction reference.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum NumericRef<'a> {
	InclusiveMinimum(&'a value::Numeric),
	ExclusiveMinimum(&'a value::Numeric),
	InclusiveMaximum(&'a value::Numeric),
	ExclusiveMaximum(&'a value::Numeric),
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
			Self::Pattern(v) => StringBindingRef::Pattern(v)
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
	pub fn is_included_in(&self, other: &Self) -> bool {
		self.map.keys().all(|r| other.map.contains_key(r))
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
						Ok(min) => match min.into_i64() {
							Ok(min) => r.insert_min(Meta(min, causes)).map_loc_err(
								error::Description::LayoutDatatypeRestrictionIntegerConflict,
							)?,
							Err(_) => todo!(),
						},
						Err(_) => todo!(),
					},
					Numeric::ExclusiveMinimum(min) => match min.into_integer() {
						Ok(min) => match min.into_i64() {
							Ok(min) => r.insert_min(Meta(min + 1, causes)).map_loc_err(
								error::Description::LayoutDatatypeRestrictionIntegerConflict,
							)?,
							Err(_) => todo!(),
						},
						Err(_) => todo!(),
					},
					Numeric::InclusiveMaximum(max) => match max.into_integer() {
						Ok(max) => match max.into_i64() {
							Ok(max) => r.insert_max(Meta(max, causes)).map_loc_err(
								error::Description::LayoutDatatypeRestrictionIntegerConflict,
							)?,
							Err(_) => todo!(),
						},
						Err(_) => todo!(),
					},
					Numeric::ExclusiveMaximum(max) => match max.into_integer() {
						Ok(max) => match max.into_i64() {
							Ok(max) => r.insert_max(Meta(max - 1, causes)).map_loc_err(
								error::Description::LayoutDatatypeRestrictionIntegerConflict,
							)?,
							Err(_) => todo!(),
						},
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
						Ok(min) => match min.into_u64() {
							Ok(min) => r.insert_min(Meta(min, causes)).map_loc_err(
								error::Description::LayoutDatatypeRestrictionUnsignedConflict,
							)?,
							Err(_) => todo!(),
						},
						Err(_) => todo!(),
					},
					Numeric::ExclusiveMinimum(min) => match min.into_non_negative_integer() {
						Ok(min) => match min.into_u64() {
							Ok(min) => r.insert_min(Meta(min + 1, causes)).map_loc_err(
								error::Description::LayoutDatatypeRestrictionUnsignedConflict,
							)?,
							Err(_) => todo!(),
						},
						Err(_) => todo!(),
					},
					Numeric::InclusiveMaximum(max) => match max.into_non_negative_integer() {
						Ok(max) => match max.into_u64() {
							Ok(max) => r.insert_max(Meta(max, causes)).map_loc_err(
								error::Description::LayoutDatatypeRestrictionUnsignedConflict,
							)?,
							Err(_) => todo!(),
						},
						Err(_) => todo!(),
					},
					Numeric::ExclusiveMaximum(max) => match max.into_non_negative_integer() {
						Ok(max) => match max.into_u64() {
							Ok(max) => r.insert_max(Meta(max - 1, causes)).map_loc_err(
								error::Description::LayoutDatatypeRestrictionUnsignedConflict,
							)?,
							Err(_) => todo!(),
						},
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
	String(StringBindingRef<'a>)
}

pub enum NumericBindingRef<'a> {
	InclusiveMinimum(&'a value::Numeric),
	ExclusiveMinimum(&'a value::Numeric),
	InclusiveMaximum(&'a value::Numeric),
	ExclusiveMaximum(&'a value::Numeric),
}

pub enum StringBindingRef<'a> {
	Pattern(&'a RegExp),
}