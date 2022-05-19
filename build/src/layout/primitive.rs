use crate::{error, Error};
use locspan::Location;
pub use treeldr::{
	layout::{primitive::restricted, primitive::RegExp, Primitive},
	Causes, Id, MaybeSet,
};

pub mod restriction;

pub use restriction::{Restriction, Restrictions};

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
