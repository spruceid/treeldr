use crate::{vocab, Id};
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Primitive {
	/// Boolean.
	Boolean,

	/// Integer number.
	Integer,

	/// Positive integer number.
	PositiveInteger,

	/// Floating point number.
	Float,

	/// Double.
	Double,

	/// String.
	String,

	/// Time.
	Time,

	/// Date.
	Date,

	/// Date and time.
	DateTime,

	/// IRI.
	Iri,

	/// URI.
	Uri,

	/// URL.
	Url,
}

impl Primitive {
	pub fn from_name(name: &str) -> Option<Self> {
		match name {
			"boolean" => Some(Self::Boolean),
			"integer" => Some(Self::Integer),
			"unsigned" => Some(Self::PositiveInteger),
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
			Self::PositiveInteger => "unsigned",
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
		use vocab::{Term, Xsd};

		match self {
			Self::Boolean => Id::Iri(Term::Xsd(Xsd::Boolean)),
			Self::Integer => Id::Iri(Term::Xsd(Xsd::Integer)),
			Self::PositiveInteger => Id::Iri(Term::Xsd(Xsd::PositiveInteger)),
			Self::Float => Id::Iri(Term::Xsd(Xsd::Float)),
			Self::Double => Id::Iri(Term::Xsd(Xsd::Double)),
			Self::String => Id::Iri(Term::Xsd(Xsd::String)),
			Self::Time => Id::Iri(Term::Xsd(Xsd::Time)),
			Self::Date => Id::Iri(Term::Xsd(Xsd::Date)),
			Self::DateTime => Id::Iri(Term::Xsd(Xsd::DateTime)),
			Self::Iri => todo!(),
			Self::Uri => Id::Iri(Term::Xsd(Xsd::AnyUri)),
			Self::Url => todo!(),
		}
	}
}

impl fmt::Display for Primitive {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.name().fmt(f)
	}
}
