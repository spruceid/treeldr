use crate::{vocab, Id};

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
