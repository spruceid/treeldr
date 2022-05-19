use crate::{error, Error};
use locspan::Location;
use std::collections::BTreeMap;
pub use treeldr::{
	layout::{primitive::restricted, primitive::RegExp, Primitive},
	Causes, Id, MaybeSet,
};

#[derive(Clone, Debug)]
pub struct Restricted<F> {
	primitive: MaybeSet<Primitive, F>,
	restrictions: Restrictions<F>,
}

impl<F> Default for Restricted<F> {
	fn default() -> Self {
		Self {
			primitive: MaybeSet::default(),
			restrictions: Restrictions::default(),
		}
	}
}

impl<F> PartialEq for Restricted<F> {
	fn eq(&self, other: &Self) -> bool {
		self.primitive.value() == other.primitive.value()
	}
}

impl<F> Eq for Restricted<F> {}

impl<F> Restricted<F> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn unrestricted(p: Primitive, causes: impl Into<Causes<F>>) -> Self {
		Self {
			primitive: MaybeSet::new(p, causes),
			restrictions: Restrictions::default(),
		}
	}

	pub fn primitive(&self) -> &MaybeSet<Primitive, F> {
		&self.primitive
	}

	pub fn set_primitive(
		&mut self,
		id: Id,
		primitive: Primitive,
		cause: Option<Location<F>>,
	) -> Result<(), Error<F>>
	where
		F: Clone + Ord,
	{
		self.primitive.try_set(
			primitive,
			cause,
			|found, expected, found_causes, expected_causes| {
				Error::new(
					error::LayoutMismatchPrimitive {
						id,
						expected,
						found,
						because: found_causes.preferred().cloned(),
					}
					.into(),
					expected_causes.preferred().cloned(),
				)
			},
		)
	}

	pub fn restrictions(&self) -> &Restrictions<F> {
		&self.restrictions
	}

	pub fn restrictions_mut(&mut self) -> &mut Restrictions<F> {
		&mut self.restrictions
	}

	pub fn try_unify(mut self, id: Id, other: Self) -> Result<Self, Error<F>>
	where
		F: Clone + Ord,
	{
		self.primitive.try_set_opt(
			other.primitive,
			|found, expected, found_causes, expected_causes| {
				Error::new(
					error::LayoutMismatchPrimitive {
						id,
						expected,
						found,
						because: found_causes.preferred().cloned(),
					}
					.into(),
					expected_causes.preferred().cloned(),
				)
			},
		)?;

		self.restrictions.unify(other.restrictions);
		Ok(self)
	}

	pub fn build(
		self,
		id: Id,
		causes: &Causes<F>,
	) -> Result<treeldr::layout::RestrictedPrimitive, Error<F>>
	where
		F: Clone,
	{
		let primitive = self.primitive.ok_or_else(|| {
			Error::new(
				error::LayoutMissingDatatypePrimitive(id).into(),
				causes.preferred().cloned(),
			)
		})?;

		match primitive.inner() {
			Primitive::Boolean => {
				self.restrictions.build_boolean(id)?;
				Ok(treeldr::layout::RestrictedPrimitive::Boolean)
			}
			Primitive::Integer => Ok(treeldr::layout::RestrictedPrimitive::Integer(
				self.restrictions.build_integer(id)?,
			)),
			Primitive::UnsignedInteger => {
				Ok(treeldr::layout::RestrictedPrimitive::UnsignedInteger(
					self.restrictions.build_unsigned_integer(id)?,
				))
			}
			Primitive::Float => Ok(treeldr::layout::RestrictedPrimitive::Float(
				self.restrictions.build_float(id)?,
			)),
			Primitive::Double => Ok(treeldr::layout::RestrictedPrimitive::Double(
				self.restrictions.build_double(id)?,
			)),
			Primitive::String => Ok(treeldr::layout::RestrictedPrimitive::String(
				self.restrictions.build_string(id)?,
			)),
			Primitive::Time => {
				self.restrictions.build_time(id)?;
				Ok(treeldr::layout::RestrictedPrimitive::Time)
			}
			Primitive::Date => {
				self.restrictions.build_date(id)?;
				Ok(treeldr::layout::RestrictedPrimitive::Date)
			}
			Primitive::DateTime => {
				self.restrictions.build_date_time(id)?;
				Ok(treeldr::layout::RestrictedPrimitive::DateTime)
			}
			Primitive::Iri => {
				self.restrictions.build_iri(id)?;
				Ok(treeldr::layout::RestrictedPrimitive::Iri)
			}
			Primitive::Uri => {
				self.restrictions.build_uri(id)?;
				Ok(treeldr::layout::RestrictedPrimitive::Uri)
			}
			Primitive::Url => {
				self.restrictions.build_url(id)?;
				Ok(treeldr::layout::RestrictedPrimitive::Url)
			}
		}
	}
}

impl<F: Ord> From<Primitive> for Restricted<F> {
	fn from(p: Primitive) -> Self {
		Self {
			primitive: MaybeSet::new(p, None),
			restrictions: Restrictions::default(),
		}
	}
}

#[derive(Clone, Debug)]
pub struct Restrictions<F> {
	map: BTreeMap<Restriction, Causes<F>>,
}

impl<F> Default for Restrictions<F> {
	fn default() -> Self {
		Self {
			map: BTreeMap::default(),
		}
	}
}

impl<F> PartialEq for Restrictions<F> {
	fn eq(&self, other: &Self) -> bool {
		self.map.len() == other.map.len()
			&& self.map.keys().zip(other.map.keys()).all(|(a, b)| a == b)
	}
}

impl<F> Eq for Restrictions<F> {}

impl<F: Ord> Restrictions<F> {
	pub fn insert(&mut self, restriction: Restriction, causes: impl Into<Causes<F>>) {
		self.map
			.entry(restriction)
			.or_insert_with(Causes::new)
			.extend(causes.into())
	}

	pub fn unify(&mut self, other: Self) {
		for (r, causes) in other.map {
			self.insert(r, causes)
		}
	}
}

impl<F: Clone> Restrictions<F> {
	pub fn build_boolean(self, id: Id) -> Result<(), Error<F>> {
		match self.map.into_iter().next() {
			Some((restriction, causes)) => Err(Error::new(
				error::LayoutDatatypeRestrictionInvalid {
					id,
					primitive: Primitive::Boolean,
					restriction,
				}
				.into(),
				causes.preferred().cloned(),
			)),
			None => Ok(()),
		}
	}

	pub fn build_integer(self, id: Id) -> Result<restricted::integer::Restrictions, Error<F>> {
		match self.map.into_iter().next() {
			Some((restriction, causes)) => Err(Error::new(
				error::LayoutDatatypeRestrictionInvalid {
					id,
					primitive: Primitive::Integer,
					restriction,
				}
				.into(),
				causes.preferred().cloned(),
			)),
			None => Ok(restricted::integer::Restrictions::default()),
		}
	}

	pub fn build_unsigned_integer(
		self,
		id: Id,
	) -> Result<restricted::unsigned::Restrictions, Error<F>> {
		match self.map.into_iter().next() {
			Some((restriction, causes)) => Err(Error::new(
				error::LayoutDatatypeRestrictionInvalid {
					id,
					primitive: Primitive::UnsignedInteger,
					restriction,
				}
				.into(),
				causes.preferred().cloned(),
			)),
			None => Ok(restricted::unsigned::Restrictions::default()),
		}
	}

	pub fn build_float(self, id: Id) -> Result<restricted::float::Restrictions, Error<F>> {
		match self.map.into_iter().next() {
			Some((restriction, causes)) => Err(Error::new(
				error::LayoutDatatypeRestrictionInvalid {
					id,
					primitive: Primitive::Float,
					restriction,
				}
				.into(),
				causes.preferred().cloned(),
			)),
			None => Ok(restricted::float::Restrictions::default()),
		}
	}

	pub fn build_double(self, id: Id) -> Result<restricted::double::Restrictions, Error<F>> {
		match self.map.into_iter().next() {
			Some((restriction, causes)) => Err(Error::new(
				error::LayoutDatatypeRestrictionInvalid {
					id,
					primitive: Primitive::Double,
					restriction,
				}
				.into(),
				causes.preferred().cloned(),
			)),
			None => Ok(restricted::double::Restrictions::default()),
		}
	}

	pub fn build_string(self, _id: Id) -> Result<restricted::string::Restrictions, Error<F>> {
		let mut p = restricted::string::Restrictions::default();

		for (restriction, _causes) in self.map.into_iter() {
			match restriction {
				Restriction::Pattern(regexp) => p.add_pattern(regexp), // other => {
				                                                       // 	return Err(Error::new(
				                                                       // 		error::LayoutDatatypeRestrictionInvalid {
				                                                       // 			id,
				                                                       // 			primitive: Primitive::String,
				                                                       // 			restriction: other
				                                                       // 		}.into(),
				                                                       // 		causes.preferred().cloned()
				                                                       // 	))
				                                                       // }
			}
		}

		Ok(p)
	}

	pub fn build_time(self, id: Id) -> Result<(), Error<F>> {
		match self.map.into_iter().next() {
			Some((restriction, causes)) => Err(Error::new(
				error::LayoutDatatypeRestrictionInvalid {
					id,
					primitive: Primitive::Time,
					restriction,
				}
				.into(),
				causes.preferred().cloned(),
			)),
			None => Ok(()),
		}
	}

	pub fn build_date(self, id: Id) -> Result<(), Error<F>> {
		match self.map.into_iter().next() {
			Some((restriction, causes)) => Err(Error::new(
				error::LayoutDatatypeRestrictionInvalid {
					id,
					primitive: Primitive::Date,
					restriction,
				}
				.into(),
				causes.preferred().cloned(),
			)),
			None => Ok(()),
		}
	}

	pub fn build_date_time(self, id: Id) -> Result<(), Error<F>> {
		match self.map.into_iter().next() {
			Some((restriction, causes)) => Err(Error::new(
				error::LayoutDatatypeRestrictionInvalid {
					id,
					primitive: Primitive::DateTime,
					restriction,
				}
				.into(),
				causes.preferred().cloned(),
			)),
			None => Ok(()),
		}
	}

	pub fn build_iri(self, id: Id) -> Result<(), Error<F>> {
		match self.map.into_iter().next() {
			Some((restriction, causes)) => Err(Error::new(
				error::LayoutDatatypeRestrictionInvalid {
					id,
					primitive: Primitive::Iri,
					restriction,
				}
				.into(),
				causes.preferred().cloned(),
			)),
			None => Ok(()),
		}
	}

	pub fn build_uri(self, id: Id) -> Result<(), Error<F>> {
		match self.map.into_iter().next() {
			Some((restriction, causes)) => Err(Error::new(
				error::LayoutDatatypeRestrictionInvalid {
					id,
					primitive: Primitive::Uri,
					restriction,
				}
				.into(),
				causes.preferred().cloned(),
			)),
			None => Ok(()),
		}
	}

	pub fn build_url(self, id: Id) -> Result<(), Error<F>> {
		match self.map.into_iter().next() {
			Some((restriction, causes)) => Err(Error::new(
				error::LayoutDatatypeRestrictionInvalid {
					id,
					primitive: Primitive::Url,
					restriction,
				}
				.into(),
				causes.preferred().cloned(),
			)),
			None => Ok(()),
		}
	}
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Restriction {
	Pattern(RegExp),
}

// pub trait IntersectedWith<F>: Sized {
// 	fn intersected_with(
// 		self,
// 		id: Id,
// 		other: &Self,
// 		name: MaybeSet<Name, F>,
// 		cause: Option<&Location<F>>,
// 	) -> Result<Self, Error<F>>;
// }

// impl<F: Clone + Ord> IntersectedWith<F> for treeldr::layout::Literal<F> {
// 	fn intersected_with(
// 		self,
// 		id: Id,
// 		other: &Self,
// 		name: MaybeSet<Name, F>,
// 		cause: Option<&Location<F>>,
// 	) -> Result<Self, Error<F>> {
// 		let this = self.into_parts();
// 		if this.regexp == *other.regexp() {
// 			Ok(Self::new(
// 				this.regexp,
// 				name.unwrap().unwrap_or(this.name),
// 				this.should_inline && other.should_inline(),
// 			))
// 		} else {
// 			Err(Caused::new(
// 				error::LayoutIntersectionFailed { id }.into(),
// 				cause.cloned(),
// 			))
// 		}
// 	}
// }
