use crate::{error, Error};
use locspan::Meta;
use treeldr::metadata::Merge;
pub use treeldr::{
	layout::{primitive::RegExp, Primitive},
	Id, MetaOption, Metadata,
};

pub mod restriction;

pub use restriction::{Restriction, Restrictions};

pub trait BuildPrimitive<M>: Sized {
	fn try_unify(self, id: Id, other: Meta<Self, M>, meta: M) -> Result<Meta<Self, M>, Error<M>>;

	fn build(
		self,
		id: Id,
		restrictions: Restrictions<M>,
		causes: &M,
	) -> Result<treeldr::layout::RestrictedPrimitive<M>, Error<M>>;
}

impl<M: Clone + Merge> BuildPrimitive<M> for Primitive {
	fn try_unify(
		self,
		id: Id,
		Meta(other, other_meta): Meta<Self, M>,
		meta: M,
	) -> Result<Meta<Self, M>, Error<M>> {
		if self == other {
			Ok(Meta(self, meta.merged_with(other_meta)))
		} else {
			Err(Error::new(
				error::LayoutMismatchPrimitive {
					id,
					expected: self,
					found: other,
					because: meta,
				}
				.into(),
				other_meta,
			))
		}
	}

	fn build(
		self,
		id: Id,
		restrictions: Restrictions<M>,
		_causes: &M,
	) -> Result<treeldr::layout::RestrictedPrimitive<M>, Error<M>> {
		match self {
			Primitive::Boolean => {
				restrictions.build_boolean(id)?;
				Ok(treeldr::layout::RestrictedPrimitive::Boolean)
			}
			Primitive::Integer => Ok(treeldr::layout::RestrictedPrimitive::Integer(
				restrictions.build_integer(id)?,
			)),
			Primitive::UnsignedInteger => {
				Ok(treeldr::layout::RestrictedPrimitive::UnsignedInteger(
					restrictions.build_unsigned_integer(id)?,
				))
			}
			Primitive::Float => Ok(treeldr::layout::RestrictedPrimitive::Float(
				restrictions.build_float(id)?,
			)),
			Primitive::Double => Ok(treeldr::layout::RestrictedPrimitive::Double(
				restrictions.build_double(id)?,
			)),
			Primitive::String => Ok(treeldr::layout::RestrictedPrimitive::String(
				restrictions.build_string(id)?,
			)),
			Primitive::Time => {
				restrictions.build_time(id)?;
				Ok(treeldr::layout::RestrictedPrimitive::Time)
			}
			Primitive::Date => {
				restrictions.build_date(id)?;
				Ok(treeldr::layout::RestrictedPrimitive::Date)
			}
			Primitive::DateTime => {
				restrictions.build_date_time(id)?;
				Ok(treeldr::layout::RestrictedPrimitive::DateTime)
			}
			Primitive::Iri => {
				restrictions.build_iri(id)?;
				Ok(treeldr::layout::RestrictedPrimitive::Iri)
			}
			Primitive::Uri => {
				restrictions.build_uri(id)?;
				Ok(treeldr::layout::RestrictedPrimitive::Uri)
			}
			Primitive::Url => {
				restrictions.build_url(id)?;
				Ok(treeldr::layout::RestrictedPrimitive::Url)
			}
		}
	}
}
