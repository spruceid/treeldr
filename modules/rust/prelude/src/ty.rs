pub use xsd_types::{
	Base64Binary as Base64Bytes, Base64BinaryBuf as Base64BytesBuf, HexBinary as HexBytes,
	HexBinaryBuf as HexBytesBuf, Integer, NegativeInteger, NonNegativeInteger, NonPositiveInteger,
	PositiveInteger,
};

pub type Bytes = [u8];
pub type BytesBuf = Vec<u8>;

pub type Cid = str;
pub type CidBuf = String;
