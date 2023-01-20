mod decimal;
mod double;
mod float;
mod integer;
mod non_negative_integer;
mod rational;
mod real;

pub use decimal::Decimal;
pub use double::Double;
pub use float::Float;
pub use integer::Integer;
use locspan::Meta;
pub use non_negative_integer::NonNegativeInteger;
pub use rational::Rational;
use rdf_types::StringLiteral;
pub use real::Real;

use crate::{
	vocab::{Owl, Term, Xsd},
	IriIndex,
};

/// Numeric value.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Numeric {
	Real(Real),
	Rational(Rational),
	Decimal(Decimal),
	Integer(Integer),
	NonNegativeInteger(NonNegativeInteger),
	Float(Float),
	Double(Double),
}

pub enum Error<M> {
	UnexpectedString(StringLiteral),
	UnexpectedType(Meta<StringLiteral, M>, IriIndex),
	InvalidReal,
}

impl Numeric {
	pub fn from_rdf_literal<M>(lit: crate::vocab::Literal<M>) -> Result<Self, Meta<Error<M>, M>> {
		match lit {
			crate::vocab::Literal::String(Meta(s, meta)) => {
				Err(Meta(Error::UnexpectedString(s), meta))
			}
			crate::vocab::Literal::LangString(Meta(s, meta), _) => {
				Err(Meta(Error::UnexpectedString(s), meta))
			}
			crate::vocab::Literal::TypedString(s, Meta(ty, meta)) => match ty {
				IriIndex::Iri(Term::Owl(Owl::Real)) => Err(Meta(Error::InvalidReal, meta)),
				IriIndex::Iri(Term::Owl(Owl::Rational)) => {
					todo!("OWL rational")
				}
				IriIndex::Iri(Term::Xsd(Xsd::Decimal)) => {
					todo!("XSD decimal")
				}
				IriIndex::Iri(Term::Xsd(Xsd::Integer)) => {
					todo!("XSD integer")
				}
				IriIndex::Iri(Term::Xsd(Xsd::NonNegativeInteger)) => {
					todo!("XSD non negative integer")
				}
				IriIndex::Iri(Term::Xsd(Xsd::Float)) => {
					todo!("XSD float")
				}
				IriIndex::Iri(Term::Xsd(Xsd::Double)) => {
					todo!("XSD double")
				}
				_ => Err(Meta(Error::UnexpectedType(s, ty), meta)),
			},
		}
	}

	pub fn into_real(self) -> Result<Real, Self> {
		match self {
			Self::Real(r) => Ok(r),
			Self::Rational(r) => Ok(r.into()),
			Self::Decimal(d) => Ok(d.into()),
			Self::Integer(i) => Ok(i.into()),
			Self::NonNegativeInteger(i) => Ok(i.into()),
			other => Err(other),
		}
	}

	pub fn into_rational(self) -> Result<Rational, Self> {
		match self {
			Self::Real(r) => r.try_into().map_err(Self::Real),
			Self::Rational(r) => Ok(r),
			Self::Decimal(d) => Ok(d.into()),
			Self::Integer(i) => Ok(i.into()),
			Self::NonNegativeInteger(i) => Ok(i.into()),
			other => Err(other),
		}
	}

	pub fn into_decimal(self) -> Result<Decimal, Self> {
		match self {
			Self::Real(r) => r.try_into().map_err(Self::Real),
			Self::Rational(r) => r.try_into().map_err(Self::Rational),
			Self::Decimal(d) => Ok(d),
			Self::Integer(i) => Ok(i.into()),
			Self::NonNegativeInteger(i) => Ok(i.into()),
			other => Err(other),
		}
	}

	pub fn into_integer(self) -> Result<Integer, Self> {
		match self {
			Self::Real(r) => r.try_into().map_err(Self::Real),
			Self::Rational(r) => r.try_into().map_err(Self::Rational),
			Self::Decimal(d) => d.try_into().map_err(Self::Decimal),
			Self::Integer(i) => Ok(i),
			Self::NonNegativeInteger(i) => Ok(i.into()),
			other => Err(other),
		}
	}

	pub fn into_non_negative_integer(self) -> Result<NonNegativeInteger, Self> {
		match self {
			Self::Real(r) => r.try_into().map_err(Self::Real),
			Self::Rational(r) => r.try_into().map_err(Self::Rational),
			Self::Decimal(d) => d.try_into().map_err(Self::Decimal),
			Self::Integer(i) => i.try_into().map_err(Self::Integer),
			Self::NonNegativeInteger(i) => Ok(i),
			other => Err(other),
		}
	}

	pub fn into_float(self) -> Result<Float, Self> {
		match self {
			Self::Float(f) => Ok(f),
			other => Err(other),
		}
	}

	pub fn into_double(self) -> Result<Double, Self> {
		match self {
			Self::Double(d) => Ok(d),
			other => Err(other),
		}
	}
}
