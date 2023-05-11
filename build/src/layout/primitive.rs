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
			Primitive::Boolean => Ok(treeldr::layout::RestrictedPrimitive::Boolean(
				restrictions.try_map(|r| r.build(id))?,
			)),
			Primitive::Integer => Ok(treeldr::layout::RestrictedPrimitive::Integer(
				restrictions.try_map(|r| r.build(id))?,
			)),
			Primitive::NonNegativeInteger => {
				Ok(treeldr::layout::RestrictedPrimitive::NonNegativeInteger(
					restrictions.try_map(|r| r.build(id))?,
				))
			}
			Primitive::NonPositiveInteger => {
				Ok(treeldr::layout::RestrictedPrimitive::NonPositiveInteger(
					restrictions.try_map(|r| r.build(id))?,
				))
			}
			Primitive::PositiveInteger => {
				Ok(treeldr::layout::RestrictedPrimitive::PositiveInteger(
					restrictions.try_map(|r| r.build(id))?,
				))
			}
			Primitive::NegativeInteger => {
				Ok(treeldr::layout::RestrictedPrimitive::NegativeInteger(
					restrictions.try_map(|r| r.build(id))?,
				))
			}
			Primitive::I64 => Ok(treeldr::layout::RestrictedPrimitive::I64(
				restrictions.try_map(|r| r.build(id))?,
			)),
			Primitive::I32 => Ok(treeldr::layout::RestrictedPrimitive::I32(
				restrictions.try_map(|r| r.build(id))?,
			)),
			Primitive::I16 => Ok(treeldr::layout::RestrictedPrimitive::I16(
				restrictions.try_map(|r| r.build(id))?,
			)),
			Primitive::I8 => Ok(treeldr::layout::RestrictedPrimitive::I8(
				restrictions.try_map(|r| r.build(id))?,
			)),
			Primitive::U64 => Ok(treeldr::layout::RestrictedPrimitive::U64(
				restrictions.try_map(|r| r.build(id))?,
			)),
			Primitive::U32 => Ok(treeldr::layout::RestrictedPrimitive::U32(
				restrictions.try_map(|r| r.build(id))?,
			)),
			Primitive::U16 => Ok(treeldr::layout::RestrictedPrimitive::U16(
				restrictions.try_map(|r| r.build(id))?,
			)),
			Primitive::U8 => Ok(treeldr::layout::RestrictedPrimitive::U8(
				restrictions.try_map(|r| r.build(id))?,
			)),
			Primitive::Float => Ok(treeldr::layout::RestrictedPrimitive::Float(
				restrictions.try_map(|r| r.build(id))?,
			)),
			Primitive::Double => Ok(treeldr::layout::RestrictedPrimitive::Double(
				restrictions.try_map(|r| r.build(id))?,
			)),
			Primitive::Base64Bytes => Ok(treeldr::layout::RestrictedPrimitive::Base64Bytes(
				restrictions.try_map(|r| r.build(id))?,
			)),
			Primitive::HexBytes => Ok(treeldr::layout::RestrictedPrimitive::HexBytes(
				restrictions.try_map(|r| r.build(id))?,
			)),
			Primitive::String => Ok(treeldr::layout::RestrictedPrimitive::String(
				restrictions.try_map(|r| r.build(id))?,
			)),
			Primitive::Time => Ok(treeldr::layout::RestrictedPrimitive::Time(
				restrictions.try_map(|r| r.build(id))?,
			)),
			Primitive::Date => Ok(treeldr::layout::RestrictedPrimitive::Date(
				restrictions.try_map(|r| r.build(id))?,
			)),
			Primitive::DateTime => Ok(treeldr::layout::RestrictedPrimitive::DateTime(
				restrictions.try_map(|r| r.build(id))?,
			)),
			Primitive::Iri => Ok(treeldr::layout::RestrictedPrimitive::Iri(
				restrictions.try_map(|r| r.build(id))?,
			)),
			Primitive::Uri => Ok(treeldr::layout::RestrictedPrimitive::Uri(
				restrictions.try_map(|r| r.build(id))?,
			)),
			Primitive::Url => Ok(treeldr::layout::RestrictedPrimitive::Url(
				restrictions.try_map(|r| r.build(id))?,
			)),
			Primitive::Bytes => Ok(treeldr::layout::RestrictedPrimitive::Bytes(
				restrictions.try_map(|r| r.build(id))?,
			)),
			Primitive::Cid => Ok(treeldr::layout::RestrictedPrimitive::Cid(
				restrictions.try_map(|r| r.build(id))?,
			)),
		}
	}
}
