use crate::{error, Error};
use locspan::Meta;
use treeldr::metadata::Merge;
pub use treeldr::{
	layout::{primitive::restricted, primitive::RegExp, Primitive},
	Id, MetaOption, Metadata,
};

pub mod restriction;

pub use restriction::{Restriction, Restrictions};

#[derive(Clone, Debug)]
pub struct Restricted<M> {
	primitive: MetaOption<Primitive, M>,
	restrictions: Restrictions<M>,
}

impl<M> Default for Restricted<M> {
	fn default() -> Self {
		Self {
			primitive: MetaOption::default(),
			restrictions: Restrictions::default(),
		}
	}
}

impl<M> PartialEq for Restricted<M> {
	fn eq(&self, other: &Self) -> bool {
		self.primitive.value() == other.primitive.value()
	}
}

impl<M> Eq for Restricted<M> {}

impl<M> Restricted<M> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn unrestricted(p: Primitive, causes: M) -> Self {
		Self {
			primitive: MetaOption::new(p, causes),
			restrictions: Restrictions::default(),
		}
	}

	pub fn primitive(&self) -> &MetaOption<Primitive, M> {
		&self.primitive
	}

	pub fn set_primitive(
		&mut self,
		id: Id,
		primitive: Primitive,
		cause: M,
	) -> Result<(), Error<M>> {
		self.primitive.try_set(
			primitive,
			cause,
			|Meta(found, found_meta), Meta(expected, expected_meta)| {
				Error::new(
					error::LayoutMismatchPrimitive {
						id,
						expected,
						found,
						because: found_meta,
					}
					.into(),
					expected_meta,
				)
			},
		)
	}

	pub fn restrictions(&self) -> &Restrictions<M> {
		&self.restrictions
	}

	pub fn restrictions_mut(&mut self) -> &mut Restrictions<M> {
		&mut self.restrictions
	}

	pub fn try_unify(mut self, id: Id, other: Self) -> Result<Self, Error<M>>
	where
		M: Merge,
	{
		self.primitive.try_set_opt(
			other.primitive,
			|Meta(found, found_meta), Meta(expected, expected_meta)| {
				Error::new(
					error::LayoutMismatchPrimitive {
						id,
						expected,
						found,
						because: found_meta,
					}
					.into(),
					expected_meta,
				)
			},
		)?;

		self.restrictions.unify(other.restrictions);
		Ok(self)
	}

	pub fn is_included_in(&self, other: &Self) -> bool {
		self.primitive().value() == other.primitive().value()
			&& self.restrictions.is_included_in(&other.restrictions)
	}

	pub fn build(self, id: Id, causes: &M) -> Result<treeldr::layout::RestrictedPrimitive, Error<M>>
	where
		M: Clone,
	{
		let primitive = self.primitive.ok_or_else(|| {
			Error::new(
				error::LayoutMissingDatatypePrimitive(id).into(),
				causes.clone(),
			)
		})?;

		match primitive.value() {
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

impl<M: Default> From<Primitive> for Restricted<M> {
	fn from(p: Primitive) -> Self {
		Self {
			primitive: MetaOption::new(p, M::default()),
			restrictions: Restrictions::default(),
		}
	}
}

impl<M> From<Meta<Primitive, M>> for Restricted<M> {
	fn from(Meta(p, m): Meta<Primitive, M>) -> Self {
		Self {
			primitive: MetaOption::new(p, m),
			restrictions: Restrictions::default(),
		}
	}
}
