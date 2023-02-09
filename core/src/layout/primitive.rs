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

	/// Unsigned integer number.
	#[iri("tldr:UnsignedInteger")]
	UnsignedInteger,

	/// Floating point number.
	#[iri("tldr:Float")]
	Float,

	/// Double.
	#[iri("tldr:Double")]
	Double,

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
}

impl Primitive {
	pub fn from_id(id: Id) -> Option<Self> {
		use vocab::{Term, TreeLdr};
		match id {
			Id::Iri(IriIndex::Iri(Term::TreeLdr(TreeLdr::Primitive(p)))) => Some(p),
			_ => None,
		}
	}

	pub fn from_name(name: &str) -> Option<Self> {
		match name {
			"boolean" => Some(Self::Boolean),
			"integer" => Some(Self::Integer),
			"unsigned" => Some(Self::UnsignedInteger),
			"float" => Some(Self::Float),
			"double" => Some(Self::Double),
			"string" => Some(Self::String),
			"time" => Some(Self::Time),
			"date" => Some(Self::Date),
			"datetime" => Some(Self::DateTime),
			"iri" => Some(Self::Iri),
			"uri" => Some(Self::Uri),
			"url" => Some(Self::Url),
			_ => None,
		}
	}

	pub fn name(&self) -> &'static str {
		match self {
			Self::Boolean => "boolean",
			Self::Integer => "integer",
			Self::UnsignedInteger => "unsigned",
			Self::Float => "float",
			Self::Double => "double",
			Self::String => "string",
			Self::Time => "time",
			Self::Date => "date",
			Self::DateTime => "datetime",
			Self::Iri => "iri",
			Self::Uri => "uri",
			Self::Url => "url",
		}
	}

	pub fn id(&self) -> Id {
		use vocab::{Term, TreeLdr};
		Id::Iri(IriIndex::Iri(Term::TreeLdr(TreeLdr::Primitive(*self))))
	}

	pub fn ty(&self) -> TId<crate::Type> {
		TId::new(self.id())
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