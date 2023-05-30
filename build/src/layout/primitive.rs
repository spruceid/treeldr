use super::restriction::primitive::Restrictions;
use crate::{Error, error};
use locspan::Meta;
use treeldr::{metadata::Merge, layout::primitive::{Derived, DerivedFrom, PrimitiveLayoutType}};
pub use treeldr::{
	layout::{primitive::RegExp, Primitive},
	Id, MetaOption, Metadata,
};
use xsd_types::{Boolean, Integer, NonNegativeInteger, NonPositiveInteger, PositiveInteger, NegativeInteger, UnsignedLong, UnsignedInt, UnsignedShort, UnsignedByte, Long, Int, Short, Byte, Float, Double, Base64BinaryBuf, HexBinaryBuf};

macro_rules! import_literals {
	( $($f:ident ($m:ident) : $ty:ty),* ) => {
		pub trait ImportOptionalLiteral<M> {
			$(
				fn $f (self) -> Result<treeldr::FunctionalPropertyValue<$ty, M>, Error<M>>;
			)*
		}

		impl<M: Clone> ImportOptionalLiteral<M> for treeldr::FunctionalPropertyValue<treeldr::value::Literal, M> {
			$(
				fn $f (self) -> Result<treeldr::FunctionalPropertyValue<$ty, M>, Error<M>> {
					self.try_map_borrow_metadata(|v, m| v.$m().map_err(|value| Meta(
						error::LiteralTypeMismatch {
							value,
							expected_type: <$ty as PrimitiveLayoutType>::PRIMITIVE.id()
						}.into(),
						m.first().unwrap().value.into_metadata().clone()
					)))
				}
			)*
		}
	};
}

import_literals! {
	into_optional_boolean (into_boolean) : Boolean,
	into_optional_integer (into_integer) : Integer,
	into_optional_non_negative_integer (into_non_negative_integer) : NonNegativeInteger,
	into_optional_non_positive_integer (into_non_positive_integer) : NonPositiveInteger,
	into_optional_positive_integer (into_positive_integer) : PositiveInteger,
	into_optional_negative_integer (into_negative_integer) : NegativeInteger,
	into_optional_unsigned_long (into_unsigned_long) : UnsignedLong,
	into_optional_unsigned_int (into_unsigned_int) : UnsignedInt,
	into_optional_unsigned_short (into_unsigned_short) : UnsignedShort,
	into_optional_unsigned_byte (into_unsigned_byte) : UnsignedByte,
	into_optional_long (into_long) : Long,
	into_optional_int (into_int) : Int,
	into_optional_short (into_short) : Short,
	into_optional_byte (into_byte) : Byte,
	into_optional_float (into_float) : Float,
	into_optional_double (into_double) : Double,
	into_optional_string (into_string) : String,
	into_optional_base64_binary (into_base64_binary) : Base64BinaryBuf,
	into_optional_hex_binary (into_hex_binary) : HexBinaryBuf
}

pub trait BuildPrimitive<M>: Sized {
	fn build(
		self,
		id: Id,
		restrictions: MetaOption<Restrictions<M>, M>,
		default: treeldr::FunctionalPropertyValue<treeldr::value::Literal, M>
	) -> Result<treeldr::layout::primitive::Derived<M>, Error<M>>;
}

impl<M: Clone + Merge> BuildPrimitive<M> for Primitive {
	fn build(
		self,
		id: Id,
		restrictions: MetaOption<Restrictions<M>, M>,
		default: treeldr::FunctionalPropertyValue<treeldr::value::Literal, M>
	) -> Result<treeldr::layout::primitive::Derived<M>, Error<M>> {
		match self {
			Primitive::Boolean => Ok(Derived::Boolean(
				DerivedFrom::new(
					restrictions.try_map(|r| r.build(id))?,
					default.into_optional_boolean()?
				)
			)),
			Primitive::Integer => Ok(Derived::Integer(
				DerivedFrom::new(
					restrictions.try_map(|r| r.build(id))?,
					default.into_optional_integer()?
				)
			)),
			Primitive::NonNegativeInteger => Ok(Derived::NonNegativeInteger(
				DerivedFrom::new(
					restrictions.try_map(|r| r.build(id))?,
					default.into_optional_non_negative_integer()?
				)
			)),
			Primitive::NonPositiveInteger => Ok(Derived::NonPositiveInteger(
				DerivedFrom::new(
					restrictions.try_map(|r| r.build(id))?,
					default.into_optional_non_positive_integer()?
				)
			)),
			Primitive::PositiveInteger => Ok(Derived::PositiveInteger(
				DerivedFrom::new(
					restrictions.try_map(|r| r.build(id))?,
					default.into_optional_positive_integer()?
				)
			)),
			Primitive::NegativeInteger => Ok(Derived::NegativeInteger(
				DerivedFrom::new(
					restrictions.try_map(|r| r.build(id))?,
					default.into_optional_negative_integer()?
				)
			)),
			Primitive::I64 => Ok(Derived::I64(
				DerivedFrom::new(
					restrictions.try_map(|r| r.build(id))?,
					default.into_optional_long()?
				)
			)),
			Primitive::I32 => Ok(Derived::I32(
				DerivedFrom::new(
					restrictions.try_map(|r| r.build(id))?,
					default.into_optional_int()?
				)
			)),
			Primitive::I16 => Ok(Derived::I16(
				DerivedFrom::new(
					restrictions.try_map(|r| r.build(id))?,
					default.into_optional_short()?
				)
			)),
			Primitive::I8 => Ok(Derived::I8(
				DerivedFrom::new(
					restrictions.try_map(|r| r.build(id))?,
					default.into_optional_byte()?
				)
			)),
			Primitive::U64 => Ok(Derived::U64(
				DerivedFrom::new(
					restrictions.try_map(|r| r.build(id))?,
					default.into_optional_unsigned_long()?
				)
			)),
			Primitive::U32 => Ok(Derived::U32(
				DerivedFrom::new(
					restrictions.try_map(|r| r.build(id))?,
					default.into_optional_unsigned_int()?
				)
			)),
			Primitive::U16 => Ok(Derived::U16(
				DerivedFrom::new(
					restrictions.try_map(|r| r.build(id))?,
					default.into_optional_unsigned_short()?
				)
			)),
			Primitive::U8 => Ok(Derived::U8(
				DerivedFrom::new(
					restrictions.try_map(|r| r.build(id))?,
					default.into_optional_unsigned_byte()?
				)
			)),
			Primitive::F32 => Ok(Derived::F32(
				DerivedFrom::new(
					restrictions.try_map(|r| r.build(id))?,
					default.into_optional_float()?
				)
			)),
			Primitive::F64 => Ok(Derived::F64(
				DerivedFrom::new(
					restrictions.try_map(|r| r.build(id))?,
					default.into_optional_double()?
				)
			)),
			Primitive::Base64BytesBuf => Ok(Derived::Base64BytesBuf(
				DerivedFrom::new(
					restrictions.try_map(|r| r.build(id))?,
					default.into_optional_base64_binary()?
				)
			)),
			Primitive::HexBytesBuf => Ok(Derived::HexBytesBuf(
				DerivedFrom::new(
					restrictions.try_map(|r| r.build(id))?,
					default.into_optional_hex_binary()?
				)
			)),
			Primitive::String => Ok(Derived::String(
				DerivedFrom::new(
					restrictions.try_map(|r| r.build(id))?,
					default.into_optional_string()?
				)
			)),
			Primitive::Time => Ok(Derived::Time(
				DerivedFrom::new(
					restrictions.try_map(|r| r.build(id))?,
					default.try_map_borrow_metadata(|_, _| unimplemented!("time default"))?
				)
			)),
			Primitive::Date => Ok(Derived::Date(
				DerivedFrom::new(
					restrictions.try_map(|r| r.build(id))?,
					default.try_map_borrow_metadata(|_, _| unimplemented!("date default"))?
				)
			)),
			Primitive::DateTime => Ok(Derived::DateTime(
				DerivedFrom::new(
					restrictions.try_map(|r| r.build(id))?,
					default.try_map_borrow_metadata(|_, _| unimplemented!("datetime default"))?
				)
			)),
			Primitive::IriBuf => Ok(Derived::IriBuf(
				DerivedFrom::new(
					restrictions.try_map(|r| r.build(id))?,
					default.try_map_borrow_metadata(|_, _| unimplemented!("IRI default"))?
				)
			)),
			Primitive::UriBuf => Ok(Derived::UriBuf(
				DerivedFrom::new(
					restrictions.try_map(|r| r.build(id))?,
					default.try_map_borrow_metadata(|_, _| unimplemented!("URI default"))?
				)
			)),
			Primitive::UrlBuf => Ok(Derived::UrlBuf(
				DerivedFrom::new(
					restrictions.try_map(|r| r.build(id))?,
					default.try_map_borrow_metadata(|_, _| unimplemented!("URL default"))?
				)
			)),
			Primitive::BytesBuf => Ok(Derived::BytesBuf(
				DerivedFrom::new(
					restrictions.try_map(|r| r.build(id))?,
					default.try_map_borrow_metadata(|_, _| unimplemented!("bytes default"))?
				)
			)),
			Primitive::CidBuf => Ok(Derived::CidBuf(
				DerivedFrom::new(
					restrictions.try_map(|r| r.build(id))?,
					default.try_map_borrow_metadata(|_, _| unimplemented!("CID default"))?
				)
			))
		}
	}
}
