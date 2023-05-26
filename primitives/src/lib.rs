pub use xsd_types::{
	Boolean,
	Base64Binary as Base64Bytes, Base64BinaryBuf as Base64BytesBuf, Byte as I8, Double as F64,
	Float as F32, HexBinary as HexBytes, HexBinaryBuf as HexBytesBuf, Int as I32, Integer,
	Long as I64, NegativeInteger, NonNegativeInteger, NonPositiveInteger, PositiveInteger,
	Short as I16, UnsignedByte as U8, UnsignedInt as U32, UnsignedLong as U64,
	UnsignedShort as U16,
	Float,
	Double,
	String,
	Time,
	Date,
	DateTime
};

pub use iref::{Iri, IriBuf, IriRef, IriRefBuf};

mod bytes;
mod uri;
mod url;
mod cid;

pub use bytes::*;
pub use uri::*;
pub use url::*;
pub use cid::*;