use crate::{vocab, Id};

pub mod regexp;
pub mod restriction;

pub use regexp::RegExp;
pub use restriction::{Restriction, Restrictions};

#[derive(Clone)]
pub enum DataType {
	Primitive(Primitive),
	Derived(Derived),
}

impl DataType {
	pub fn primitive(&self) -> Primitive {
		match self {
			Self::Primitive(p) => *p,
			Self::Derived(d) => d.primitive(),
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Primitive {
	/// `xsd:boolean`.
	Boolean,

	/// `owl:real`.
	Real,

	/// `xsd:float`.
	Float,

	/// `xsd:double`.
	Double,

	/// `xsd:string`.
	String,

	/// `xsd:date`.
	Date,

	/// `xsd:time`.
	Time,

	/// `xsd:dateTime`.
	DateTime,

	/// `xsd:duration`.
	Duration,
}

impl Primitive {
	pub fn id(&self) -> Id {
		use vocab::{Owl, Term, Xsd};
		match self {
			Self::Boolean => Id::Iri(Term::Xsd(Xsd::Boolean)),
			Self::Real => Id::Iri(Term::Owl(Owl::Real)),
			Self::Float => Id::Iri(Term::Xsd(Xsd::Float)),
			Self::Double => Id::Iri(Term::Xsd(Xsd::Double)),
			Self::String => Id::Iri(Term::Xsd(Xsd::String)),
			Self::Date => Id::Iri(Term::Xsd(Xsd::Date)),
			Self::Time => Id::Iri(Term::Xsd(Xsd::Time)),
			Self::DateTime => Id::Iri(Term::Xsd(Xsd::DateTime)),
			Self::Duration => Id::Iri(Term::Xsd(Xsd::Duration)),
		}
	}
}

#[derive(Clone)]
pub enum Derived {
	Boolean(Id),
	Real(Id, restriction::real::Restrictions),
	Float(Id, restriction::float::Restrictions),
	Double(Id, restriction::double::Restrictions),
	String(Id, restriction::string::Restrictions),
	Date(Id),
	Time(Id),
	DateTime(Id),
	Duration(Id),
}

impl Derived {
	pub fn base(&self) -> Id {
		match self {
			Self::Boolean(id) => *id,
			Self::Real(id, _) => *id,
			Self::Float(id, _) => *id,
			Self::Double(id, _) => *id,
			Self::String(id, _) => *id,
			Self::Date(id) => *id,
			Self::Time(id) => *id,
			Self::DateTime(id) => *id,
			Self::Duration(id) => *id,
		}
	}

	pub fn primitive(&self) -> Primitive {
		match self {
			Self::Boolean(_) => Primitive::Boolean,
			Self::Real(_, _) => Primitive::Real,
			Self::Float(_, _) => Primitive::Float,
			Self::Double(_, _) => Primitive::Double,
			Self::String(_, _) => Primitive::String,
			Self::Date(_) => Primitive::Date,
			Self::Time(_) => Primitive::Time,
			Self::DateTime(_) => Primitive::DateTime,
			Self::Duration(_) => Primitive::Duration,
		}
	}

	pub fn restrictions(&self) -> Restrictions {
		match self {
			Self::Real(_, r) => Restrictions::Real(r.iter()),
			Self::Float(_, r) => Restrictions::Float(r.iter()),
			Self::Double(_, r) => Restrictions::Double(r.iter()),
			Self::String(_, r) => Restrictions::String(r.iter()),
			_ => Restrictions::None,
		}
	}
}
