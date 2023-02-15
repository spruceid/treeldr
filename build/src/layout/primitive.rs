use super::restriction::primitive::Restrictions;
use crate::Error;
use treeldr::metadata::Merge;
pub use treeldr::{
	layout::{primitive::RegExp, Primitive},
	Id, MetaOption, Metadata,
};

pub trait BuildPrimitive<M>: Sized {
	fn build(
		self,
		id: Id,
		restrictions: MetaOption<Restrictions<M>, M>,
	) -> Result<treeldr::layout::RestrictedPrimitive<M>, Error<M>>;
}

impl<M: Clone + Merge> BuildPrimitive<M> for Primitive {
	fn build(
		self,
		id: Id,
		restrictions: MetaOption<Restrictions<M>, M>,
	) -> Result<treeldr::layout::RestrictedPrimitive<M>, Error<M>> {
		match self {
			Primitive::Boolean => {
				restrictions.try_map(|r| r.build_boolean(id))?;
				Ok(treeldr::layout::RestrictedPrimitive::Boolean)
			}
			Primitive::Integer => Ok(treeldr::layout::RestrictedPrimitive::Integer(
				restrictions.try_map(|r| r.build_integer(id))?,
			)),
			Primitive::UnsignedInteger => {
				Ok(treeldr::layout::RestrictedPrimitive::UnsignedInteger(
					restrictions.try_map(|r| r.build_unsigned_integer(id))?,
				))
			}
			Primitive::Float => Ok(treeldr::layout::RestrictedPrimitive::Float(
				restrictions.try_map(|r| r.build_float(id))?,
			)),
			Primitive::Double => Ok(treeldr::layout::RestrictedPrimitive::Double(
				restrictions.try_map(|r| r.build_double(id))?,
			)),
			Primitive::String => Ok(treeldr::layout::RestrictedPrimitive::String(
				restrictions.try_map(|r| r.build_string(id))?,
			)),
			Primitive::Time => {
				restrictions.try_map(|r| r.build_time(id))?;
				Ok(treeldr::layout::RestrictedPrimitive::Time)
			}
			Primitive::Date => {
				restrictions.try_map(|r| r.build_date(id))?;
				Ok(treeldr::layout::RestrictedPrimitive::Date)
			}
			Primitive::DateTime => {
				restrictions.try_map(|r| r.build_date_time(id))?;
				Ok(treeldr::layout::RestrictedPrimitive::DateTime)
			}
			Primitive::Iri => {
				restrictions.try_map(|r| r.build_iri(id))?;
				Ok(treeldr::layout::RestrictedPrimitive::Iri)
			}
			Primitive::Uri => {
				restrictions.try_map(|r| r.build_uri(id))?;
				Ok(treeldr::layout::RestrictedPrimitive::Uri)
			}
			Primitive::Url => {
				restrictions.try_map(|r| r.build_url(id))?;
				Ok(treeldr::layout::RestrictedPrimitive::Url)
			}
		}
	}
}
