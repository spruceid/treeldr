use crate::{error, resource::BindingValueRef, Error};
use locspan::{MapLocErr, Meta};
use std::collections::BTreeMap;
use treeldr::{
	layout::primitive::restriction::{template::float::FloatType, RestrainableType},
	metadata::Merge,
	prop::UnknownProperty,
	vocab::{Term, Xsd},
	IriIndex, TId,
};
pub use treeldr::{
	layout::{primitive::restriction, primitive::RegExp, Primitive},
	value, Id, MetaOption,
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

impl<T: Into<value::Numeric>> From<restriction::template::integer::Restriction<T>> for Restriction {
	fn from(value: restriction::template::integer::Restriction<T>) -> Self {
		Self::Numeric(value.into())
	}
}

impl<T: Into<value::Numeric>> From<restriction::template::float::Restriction<T>> for Restriction {
	fn from(value: restriction::template::float::Restriction<T>) -> Self {
		Self::Numeric(value.into())
	}
}

#[derive(Debug)]
pub struct Conflict<M>(pub Restriction, pub Meta<Restriction, M>);

impl<T: Into<value::Numeric>, M> From<restriction::template::integer::Conflict<T, M>>
	for Conflict<M>
{
	fn from(value: restriction::template::integer::Conflict<T, M>) -> Self {
		Self(value.0.into(), value.1.cast())
	}
}

impl<T: Into<value::Numeric>, M> From<restriction::template::float::Conflict<T, M>>
	for Conflict<M>
{
	fn from(value: restriction::template::float::Conflict<T, M>) -> Self {
		Self(value.0.into(), value.1.cast())
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
			Self::InclusiveMinimum(v) => NumericBindingRef::InclusiveMinimum(None, v),
			Self::ExclusiveMinimum(v) => NumericBindingRef::ExclusiveMinimum(None, v),
			Self::InclusiveMaximum(v) => NumericBindingRef::InclusiveMaximum(None, v),
			Self::ExclusiveMaximum(v) => NumericBindingRef::ExclusiveMaximum(None, v),
		}
	}
}

impl<T: Into<value::Numeric>> From<restriction::template::integer::Restriction<T>> for Numeric {
	fn from(value: restriction::template::integer::Restriction<T>) -> Self {
		match value {
			restriction::template::integer::Restriction::MinInclusive(v) => {
				Self::InclusiveMinimum(v.into())
			}
			restriction::template::integer::Restriction::MaxInclusive(v) => {
				Self::InclusiveMaximum(v.into())
			}
		}
	}
}

impl<T: Into<value::Numeric>> From<restriction::template::float::Restriction<T>> for Numeric {
	fn from(value: restriction::template::float::Restriction<T>) -> Self {
		use restriction::template::float::{Max, Min};
		match value {
			restriction::template::float::Restriction::Min(Min::Included(v)) => {
				Self::InclusiveMinimum(v.into())
			}
			restriction::template::float::Restriction::Min(Min::Excluded(v)) => {
				Self::ExclusiveMinimum(v.into())
			}
			restriction::template::float::Restriction::Max(Max::Included(v)) => {
				Self::InclusiveMaximum(v.into())
			}
			restriction::template::float::Restriction::Max(Max::Excluded(v)) => {
				Self::ExclusiveMaximum(v.into())
			}
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
			Self::Pattern(v) => StringBindingRef::Pattern(None, v),
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

pub trait BuildRestrictions<M>: Sized {
	fn build(restrictions: Restrictions<M>, id: Id) -> Result<Self, Error<M>>;
}

impl<T, M: Merge> BuildRestrictions<M> for restriction::template::none::Restrictions<T, M> {
	fn build(restrictions: Restrictions<M>, id: Id) -> Result<Self, Error<M>> {
		match restrictions.map.into_iter().next() {
			Some((restriction, causes)) => Err(Error::new(
				error::LayoutDatatypeRestrictionInvalid {
					id,
					primitive: Primitive::Boolean,
					restriction,
				}
				.into(),
				causes,
			)),
			None => Ok(Self::default()),
		}
	}
}

pub trait IntegerType: Sized {
	fn pred(&self) -> Option<Self>;

	fn succ(&self) -> Option<Self>;
}

macro_rules! impl_integer_type {
	{ $id:ident, $($ty:ty [$pred_test:expr] [$succ_test:expr]),* } => {
		$(
			impl IntegerType for $ty {
				fn pred(&$id) -> Option<Self> {
					if $pred_test {
						Some($id.clone() - 1)
					} else {
						None
					}
				}

				fn succ(&$id) -> Option<Self> {
					if $succ_test {
						Some($id.clone() + 1)
					} else {
						None
					}
				}
			}
		)*
	};
}

impl_integer_type! {
	self,
	xsd_types::Integer [true] [true],
	xsd_types::NonPositiveInteger [true] [!self.is_zero()],
	xsd_types::NonNegativeInteger [!self.is_zero()] [true],
	xsd_types::NegativeInteger [true] [!self.is_minus_one()],
	xsd_types::PositiveInteger [!self.is_one()] [true],
	xsd_types::Long [*self > Self::MIN] [*self < Self::MAX],
	xsd_types::Int [*self > Self::MIN] [*self < Self::MAX],
	xsd_types::Short [*self > Self::MIN] [*self < Self::MAX],
	xsd_types::Byte [*self > Self::MIN] [*self < Self::MAX],
	xsd_types::UnsignedLong [*self > Self::MIN] [*self < Self::MAX],
	xsd_types::UnsignedInt [*self > Self::MIN] [*self < Self::MAX],
	xsd_types::UnsignedShort [*self > Self::MIN] [*self < Self::MAX],
	xsd_types::UnsignedByte [*self > Self::MIN] [*self < Self::MAX]
}

impl<
		T: IntegerType
			+ RestrainableType
			+ Clone
			+ PartialOrd
			+ TryFrom<value::Numeric>
			+ Into<value::Numeric>,
		M: Clone + Merge,
	> BuildRestrictions<M> for restriction::template::integer::Restrictions<T, M>
{
	fn build(restrictions: Restrictions<M>, id: Id) -> Result<Self, Error<M>> {
		let mut r = Self::default();

		for (restriction, causes) in restrictions.map {
			match restriction.clone() {
				Restriction::Numeric(numeric_restriction) => match numeric_restriction {
					Numeric::InclusiveMinimum(min) => match T::try_from(min) {
						Ok(min) => r.insert_min(Meta(min, causes)).map_loc_err(|c| {
							error::Description::LayoutDatatypeRestrictionConflict(c.into())
						})?,
						Err(_) => todo!(),
					},
					Numeric::ExclusiveMinimum(min) => match T::try_from(min) {
						Ok(min) => match min.succ() {
							Some(min) => r.insert_min(Meta(min, causes)).map_loc_err(|c| {
								error::Description::LayoutDatatypeRestrictionConflict(c.into())
							})?,
							None => {
								return Err(Meta(
									error::LayoutDatatypeRestrictionInvalid {
										id,
										primitive: T::PRIMITIVE,
										restriction,
									}
									.into(),
									causes,
								))
							}
						},
						Err(_) => todo!(),
					},
					Numeric::InclusiveMaximum(max) => match T::try_from(max) {
						Ok(max) => r.insert_max(Meta(max, causes)).map_loc_err(|c| {
							error::Description::LayoutDatatypeRestrictionConflict(c.into())
						})?,
						Err(_) => todo!(),
					},
					Numeric::ExclusiveMaximum(max) => match T::try_from(max) {
						Ok(max) => match max.pred() {
							Some(max) => r.insert_max(Meta(max, causes)).map_loc_err(|c| {
								error::Description::LayoutDatatypeRestrictionConflict(c.into())
							})?,
							None => {
								return Err(Meta(
									error::LayoutDatatypeRestrictionInvalid {
										id,
										primitive: T::PRIMITIVE,
										restriction,
									}
									.into(),
									causes,
								))
							}
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
}

impl<
		T: FloatType + Clone + PartialOrd + TryFrom<value::Numeric> + Into<value::Numeric>,
		M: Clone + Merge,
	> BuildRestrictions<M> for restriction::template::float::Restrictions<T, M>
{
	fn build(restrictions: Restrictions<M>, id: Id) -> Result<Self, Error<M>> {
		use restriction::template::float::{Max, Min};
		let mut r = Self::default();

		for (restriction, causes) in restrictions.map {
			match restriction {
				Restriction::Numeric(restriction) => match restriction {
					Numeric::InclusiveMinimum(min) => match T::try_from(min) {
						Ok(min) => {
							r.insert_min(Meta(Min::Included(min), causes))
								.map_loc_err(|c| {
									error::Description::LayoutDatatypeRestrictionConflict(c.into())
								})?
						}
						Err(_) => todo!(),
					},
					Numeric::ExclusiveMinimum(min) => match T::try_from(min) {
						Ok(min) => {
							r.insert_min(Meta(Min::Excluded(min), causes))
								.map_loc_err(|c| {
									error::Description::LayoutDatatypeRestrictionConflict(c.into())
								})?
						}
						Err(_) => todo!(),
					},
					Numeric::InclusiveMaximum(max) => match T::try_from(max) {
						Ok(max) => {
							r.insert_max(Meta(Max::Included(max), causes))
								.map_loc_err(|c| {
									error::Description::LayoutDatatypeRestrictionConflict(c.into())
								})?
						}
						Err(_) => todo!(),
					},
					Numeric::ExclusiveMaximum(max) => match T::try_from(max) {
						Ok(max) => {
							r.insert_max(Meta(Max::Excluded(max), causes))
								.map_loc_err(|c| {
									error::Description::LayoutDatatypeRestrictionConflict(c.into())
								})?
						}
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
}

impl<M: Clone + Merge> BuildRestrictions<M> for restriction::template::string::Restrictions<M> {
	fn build(restrictions: Restrictions<M>, id: Id) -> Result<Self, Error<M>> {
		let mut p = restriction::string::Restrictions::default();

		for (restriction, causes) in restrictions.map.into_iter() {
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
}

impl<M: Clone> Restrictions<M> {
	pub fn build<R: BuildRestrictions<M>>(self, id: Id) -> Result<R, Error<M>> {
		R::build(self, id)
	}
}

#[derive(Debug)]
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

	pub fn value(&self) -> BindingValueRef<'a> {
		match self {
			Self::Numeric(b) => b.value(),
			Self::String(b) => b.value(),
		}
	}
}

#[derive(Debug)]
pub enum NumericBindingRef<'a> {
	InclusiveMinimum(Option<TId<UnknownProperty>>, &'a value::Numeric),
	ExclusiveMinimum(Option<TId<UnknownProperty>>, &'a value::Numeric),
	InclusiveMaximum(Option<TId<UnknownProperty>>, &'a value::Numeric),
	ExclusiveMaximum(Option<TId<UnknownProperty>>, &'a value::Numeric),
}

impl<'a> NumericBindingRef<'a> {
	pub fn property(&self) -> Property {
		match self {
			Self::InclusiveMinimum(p, _) => Property::InclusiveMinimum(*p),
			Self::ExclusiveMinimum(p, _) => Property::ExclusiveMinimum(*p),
			Self::InclusiveMaximum(p, _) => Property::InclusiveMaximum(*p),
			Self::ExclusiveMaximum(p, _) => Property::ExclusiveMaximum(*p),
		}
	}

	pub fn value(&self) -> BindingValueRef<'a> {
		match self {
			Self::InclusiveMinimum(_, v) => BindingValueRef::Numeric(v),
			Self::ExclusiveMinimum(_, v) => BindingValueRef::Numeric(v),
			Self::InclusiveMaximum(_, v) => BindingValueRef::Numeric(v),
			Self::ExclusiveMaximum(_, v) => BindingValueRef::Numeric(v),
		}
	}
}

#[derive(Debug)]
pub enum StringBindingRef<'a> {
	Pattern(Option<TId<UnknownProperty>>, &'a RegExp),
}

impl<'a> StringBindingRef<'a> {
	pub fn property(&self) -> Property {
		match self {
			Self::Pattern(p, _) => Property::Pattern(*p),
		}
	}

	pub fn value(&self) -> BindingValueRef<'a> {
		match self {
			Self::Pattern(_, v) => BindingValueRef::RegExp(v),
		}
	}
}
