use crate::{vocab, Id, IriIndex, TId};
use iref_enum::IriEnum;
use std::fmt;

pub mod restriction;

pub use crate::ty::data::RegExp;
pub use restriction::{
	Restricted, RestrictionRef, Restrictions, WithRestrictions, WithRestrictionsIter,
};

#[derive(IriEnum, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[iri_prefix("tldr" = "https://treeldr.org/")]
pub enum Primitive {
	/// Boolean.
	#[iri("tldr:Boolean")]
	Boolean,

	/// Integer number.
	#[iri("tldr:Integer")]
	Integer,

	/// Non negative integer number.
	#[iri("tldr:NonNegativeInteger")]
	NonNegativeInteger,

	/// Non positive integer number.
	#[iri("tldr:NonPositiveInteger")]
	NonPositiveInteger,

	/// Strictly negative integer number.
	#[iri("tldr:NegativeInteger")]
	NegativeInteger,

	/// Strictly positive integer number.
	#[iri("tldr:PositiveInteger")]
	PositiveInteger,

	/// Floating point number.
	#[iri("tldr:Float")]
	Float,

	/// Double.
	#[iri("tldr:Double")]
	Double,

	/// I64.
	#[iri("tldr:I64")]
	I64,

	/// I32.
	#[iri("tldr:I32")]
	I32,

	/// I16.
	#[iri("tldr:I16")]
	I16,

	/// I8.
	#[iri("tldr:I8")]
	I8,

	/// U64.
	#[iri("tldr:U64")]
	U64,

	/// U32.
	#[iri("tldr:U32")]
	U32,

	/// U16.
	#[iri("tldr:U16")]
	U16,

	/// U8.
	#[iri("tldr:U8")]
	U8,

	/// Base 64 byte string.
	#[iri("tldr:Base64Bytes")]
	Base64Bytes,

	/// Hex byte string.
	#[iri("tldr:HexBytes")]
	HexBytes,

	/// String.
	#[iri("tldr:String")]
	String,

	/// Time.
	#[iri("tldr:Time")]
	Time,

	/// Date.
	#[iri("tldr:Date")]
	Date,

	/// Date and time.
	#[iri("tldr:DateTime")]
	DateTime,

	/// IRI.
	#[iri("tldr:IRI")]
	Iri,

	/// URI.
	#[iri("tldr:URI")]
	Uri,

	/// URL.
	#[iri("tldr:URL")]
	Url,

	/// Arbitrary bytes.
	#[iri("tldr:Bytes")]
	Bytes,

	/// CID (Content IDentifier).
	///
	/// See <https://github.com/multiformats/cid>
	#[iri("tldr:CID")]
	Cid,
}

impl Primitive {
	pub fn from_id(id: Id) -> Option<Self> {
		use vocab::{Term, TreeLdr};
		match id {
			Id::Iri(IriIndex::Iri(Term::TreeLdr(TreeLdr::Primitive(p)))) => Some(p),
			_ => None,
		}
	}

	pub fn name(&self) -> &'static str {
		match self {
			Self::Boolean => "boolean",
			Self::Integer => "integer",
			Self::NonNegativeInteger => "non negative integer",
			Self::NonPositiveInteger => "non positive integer",
			Self::NegativeInteger => "negative integer",
			Self::PositiveInteger => "positive integer",
			Self::Float => "float",
			Self::Double => "double",
			Self::U64 => "u64",
			Self::U32 => "u32",
			Self::U16 => "u16",
			Self::U8 => "u8",
			Self::I64 => "i64",
			Self::I32 => "i32",
			Self::I16 => "i16",
			Self::I8 => "i8",
			Self::Base64Bytes => "base 64 bytes",
			Self::HexBytes => "hex bytes",
			Self::String => "string",
			Self::Time => "time",
			Self::Date => "date",
			Self::DateTime => "date and time",
			Self::Iri => "iri",
			Self::Uri => "uri",
			Self::Url => "url",
			Self::Bytes => "bytes",
			Self::Cid => "content identifier",
		}
	}

	pub fn id(&self) -> Id {
		use vocab::{Term, TreeLdr};
		Id::Iri(IriIndex::Iri(Term::TreeLdr(TreeLdr::Primitive(*self))))
	}

	pub fn layout(&self) -> TId<crate::Layout> {
		TId::new(self.id())
	}

	pub fn natural_type_term(&self) -> Option<vocab::Term> {
		use vocab::{Term, Xsd};
		match self {
			Self::Boolean => Some(Term::Xsd(Xsd::Boolean)),
			Self::Integer => Some(Term::Xsd(Xsd::Integer)),
			Self::NonNegativeInteger => Some(Term::Xsd(Xsd::NonNegativeInteger)),
			Self::NonPositiveInteger => Some(Term::Xsd(Xsd::NonPositiveInteger)),
			Self::NegativeInteger => Some(Term::Xsd(Xsd::NegativeInteger)),
			Self::PositiveInteger => Some(Term::Xsd(Xsd::PositiveInteger)),
			Self::Float => Some(Term::Xsd(Xsd::Float)),
			Self::Double => Some(Term::Xsd(Xsd::Double)),
			Self::U64 => Some(Term::Xsd(Xsd::UnsignedLong)),
			Self::U32 => Some(Term::Xsd(Xsd::UnsignedInt)),
			Self::U16 => Some(Term::Xsd(Xsd::UnsignedShort)),
			Self::U8 => Some(Term::Xsd(Xsd::UnsignedByte)),
			Self::I64 => Some(Term::Xsd(Xsd::Long)),
			Self::I32 => Some(Term::Xsd(Xsd::Int)),
			Self::I16 => Some(Term::Xsd(Xsd::Short)),
			Self::I8 => Some(Term::Xsd(Xsd::Byte)),
			Self::Base64Bytes => Some(Term::Xsd(Xsd::Base64Binary)),
			Self::HexBytes => Some(Term::Xsd(Xsd::HexBinary)),
			Self::String => Some(Term::Xsd(Xsd::String)),
			Self::Time => Some(Term::Xsd(Xsd::Time)),
			Self::Date => Some(Term::Xsd(Xsd::Date)),
			Self::DateTime => Some(Term::Xsd(Xsd::DateTime)),
			Self::Iri => Some(Term::Xsd(Xsd::AnyUri)),
			Self::Uri => Some(Term::Xsd(Xsd::AnyUri)),
			Self::Url => Some(Term::Xsd(Xsd::AnyUri)),
			Self::Bytes => None,
			Self::Cid => None,
		}
	}

	pub fn natural_type(&self) -> Option<TId<crate::Type>> {
		self.natural_type_term()
			.map(|t| TId::new(Id::Iri(IriIndex::Iri(t))))
	}
}

impl fmt::Display for Primitive {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.name().fmt(f)
	}
}

impl From<Primitive> for Id {
	fn from(value: Primitive) -> Self {
		value.id()
	}
}
