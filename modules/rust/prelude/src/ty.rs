pub use xsd_types::{
	Base64Binary as Base64Bytes, Base64BinaryBuf as Base64BytesBuf, Byte as I8, Double as F64,
	Float as F32, HexBinary as HexBytes, HexBinaryBuf as HexBytesBuf, Int as I32, Integer,
	Long as I64, NegativeInteger, NonNegativeInteger, NonPositiveInteger, PositiveInteger,
	Short as I16, UnsignedByte as U8, UnsignedInt as U32, UnsignedLong as U64,
	UnsignedShort as U16,
};

pub type Bytes = [u8];
pub type BytesBuf = Vec<u8>;

pub type Cid = str;
pub type CidBuf = String;
