pub use xsd_types::{
	Base64Binary as Base64Bytes, Base64BinaryBuf as Base64BytesBuf, Boolean, Byte as I8, Date,
	DateTime, Double as F64, Double, Float as F32, Float, HexBinary as HexBytes,
	HexBinaryBuf as HexBytesBuf, Int as I32, Integer, Long as I64, NegativeInteger,
	NonNegativeInteger, NonPositiveInteger, PositiveInteger, Short as I16, String, Time,
	UnsignedByte as U8, UnsignedInt as U32, UnsignedLong as U64, UnsignedShort as U16,
};

pub use iref::{Iri, IriBuf, IriRef, IriRefBuf};

mod bytes;
mod cid;
mod uri;
mod url;

pub use bytes::*;
pub use cid::*;
pub use uri::*;
pub use url::*;
