use super::Primitive;
use locspan::Meta;

pub mod double;
pub mod float;
pub mod integer;
pub mod string;
pub mod unsigned;

/// Restricted primitive layout.
#[derive(Clone, Debug)]
pub enum Restricted<M> {
	Boolean,
	Integer(integer::Restrictions<M>),
	UnsignedInteger(unsigned::Restrictions<M>),
	Float(float::Restrictions<M>),
	Double(double::Restrictions<M>),
	String(string::Restrictions<M>),
	Time,
	Date,
	DateTime,
	Iri,
	Uri,
	Url,
}

impl<M> Restricted<M> {
	pub fn primitive(&self) -> Primitive {
		match self {
			Self::Boolean => Primitive::Boolean,
			Self::Integer(_) => Primitive::Integer,
			Self::UnsignedInteger(_) => Primitive::UnsignedInteger,
			Self::Float(_) => Primitive::Float,
			Self::Double(_) => Primitive::Double,
			Self::String(_) => Primitive::String,
			Self::Time => Primitive::Time,
			Self::Date => Primitive::Date,
			Self::DateTime => Primitive::DateTime,
			Self::Iri => Primitive::Iri,
			Self::Uri => Primitive::Uri,
			Self::Url => Primitive::Url,
		}
	}

	pub fn is_restricted(&self) -> bool {
		match self {
			Self::Integer(r) => r.is_restricted(),
			Self::UnsignedInteger(r) => r.is_restricted(),
			Self::Float(r) => r.is_restricted(),
			Self::Double(r) => r.is_restricted(),
			Self::String(r) => r.is_restricted(),
			_ => false,
		}
	}

	pub fn restrictions(&self) -> Restrictions<M> {
		match self {
			Self::Integer(r) => Restrictions::Integer(r.iter()),
			Self::UnsignedInteger(r) => Restrictions::UnsignedInteger(r.iter()),
			Self::Float(r) => Restrictions::Float(r.iter()),
			Self::Double(r) => Restrictions::Double(r.iter()),
			Self::String(r) => Restrictions::String(r.iter()),
			_ => Restrictions::None,
		}
	}
}

impl<M> From<Primitive> for Restricted<M> {
	fn from(p: Primitive) -> Self {
		match p {
			Primitive::Boolean => Self::Boolean,
			Primitive::Integer => Self::Integer(integer::Restrictions::default()),
			Primitive::UnsignedInteger => Self::UnsignedInteger(unsigned::Restrictions::default()),
			Primitive::Float => Self::Float(float::Restrictions::default()),
			Primitive::Double => Self::Double(double::Restrictions::default()),
			Primitive::String => Self::String(string::Restrictions::default()),
			Primitive::Time => Self::Time,
			Primitive::Date => Self::Date,
			Primitive::DateTime => Self::DateTime,
			Primitive::Iri => Self::Iri,
			Primitive::Uri => Self::Uri,
			Primitive::Url => Self::Url,
		}
	}
}

pub enum RestrictionRef<'a> {
	Integer(integer::Restriction),
	UnsignedInteger(unsigned::Restriction),
	Float(float::Restriction),
	Double(double::Restriction),
	String(string::RestrictionRef<'a>),
}

pub enum Restrictions<'a, M> {
	None,
	Integer(integer::Iter<'a, M>),
	UnsignedInteger(unsigned::Iter<'a, M>),
	Float(float::Iter<'a, M>),
	Double(double::Iter<'a, M>),
	String(string::Iter<'a, M>),
}

impl<'a, M> Iterator for Restrictions<'a, M> {
	type Item = Meta<RestrictionRef<'a>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::None => None,
			Self::Integer(r) => r.next().map(|r| r.map(RestrictionRef::Integer)),
			Self::UnsignedInteger(r) => r.next().map(|r| r.map(RestrictionRef::UnsignedInteger)),
			Self::Float(r) => r.next().map(|r| r.map(RestrictionRef::Float)),
			Self::Double(r) => r.next().map(|r| r.map(RestrictionRef::Double)),
			Self::String(r) => r.next().map(|r| r.map(RestrictionRef::String)),
		}
	}
}
