use crate::vocab::{self, Xsd};

pub mod double;
pub mod float;
pub mod real;
pub mod string;

pub enum Restriction<'a> {
	Real(real::Restriction<'a>),
	Float(float::Restriction),
	Double(double::Restriction),
	String(string::Restriction<'a>),
}

pub enum Restrictions<'a> {
	None,
	Real(real::Iter<'a>),
	Float(float::Iter),
	Double(double::Iter),
	String(string::Iter<'a>),
}

impl<'a> Iterator for Restrictions<'a> {
	type Item = Restriction<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::None => None,
			Self::Real(r) => r.next().map(Restriction::Real),
			Self::Float(r) => r.next().map(Restriction::Float),
			Self::Double(r) => r.next().map(Restriction::Double),
			Self::String(r) => r.next().map(Restriction::String),
		}
	}
}

// #[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
// pub enum Numeric<T> {
// 	MinInclusive(T),
// 	MinExclusive(T),
// 	MaxInclusive(T),
// 	MaxExclusive(T),
// }

// impl<T> Numeric<T> {
// 	pub fn as_binding(&self) -> NumericBindingRef<T> {
// 		match self {
// 			Self::MinInclusive(v) => NumericBindingRef::MinInclusive(v),
// 			Self::MinExclusive(v) => NumericBindingRef::MinExclusive(v),
// 			Self::MaxInclusive(v) => NumericBindingRef::MaxInclusive(v),
// 			Self::MaxExclusive(v) => NumericBindingRef::MaxExclusive(v)
// 		}
// 	}
// }

// #[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
// pub enum String {
// 	MinLength(value::Integer),
// 	MaxLength(value::Integer),
// 	Pattern(RegExp),
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Property {
	MinInclusive,
	MinExclusive,
	MaxInclusive,
	MaxExclusive,
	MinLength,
	MaxLength,
	Pattern
}

impl Property {
	pub fn term(&self) -> vocab::Term {
		match self {
			Self::MinInclusive => vocab::Term::Xsd(Xsd::MinInclusive),
			Self::MinExclusive => vocab::Term::Xsd(Xsd::MinExclusive),
			Self::MaxInclusive => vocab::Term::Xsd(Xsd::MaxInclusive),
			Self::MaxExclusive => vocab::Term::Xsd(Xsd::MaxExclusive),
			Self::MinLength => vocab::Term::Xsd(Xsd::MinLength),
			Self::MaxLength => vocab::Term::Xsd(Xsd::MaxLength),
			Self::Pattern => vocab::Term::Xsd(Xsd::Pattern),
		}
	}

	pub fn name(&self) -> &'static str {
		match self {
			Self::MinInclusive => "inclusive minimum",
			Self::MinExclusive => "exclusive minimum",
			Self::MaxInclusive => "inclusive maximum",
			Self::MaxExclusive => "exclusive maximum",
			Self::MinLength => "minimum length",
			Self::MaxLength => "maximum length",
			Self::Pattern => "pattern"
		}
	}
}
