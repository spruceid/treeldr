use btree_range_map::RangeSet;
use ordered_float::NotNan;
use crate::{
	Id,
	value
};

pub mod regexp;

pub use regexp::RegExp;

#[derive(Clone)]
pub enum DataType {
	Primitive(Primitive),
	Derived(Derived)
}

#[derive(Clone)]
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
	Duration
}

#[derive(Clone)]
pub enum Derived {
	Boolean(Id),
	Real(Id, RealRestrictions),
	Float(Id, FloatRestrictions),
	Double(Id, DoubleRestrictions),
	String(Id, StringRestrictions),
	Date(Id),
	Time(Id),
	DateTime(Id),
	Duration(Id)
}

impl Derived {
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
}

#[derive(Clone)]
pub struct RealRestrictions {
	bounds: RangeSet<value::Real>
}

impl RealRestrictions {
	pub fn bounds(&self) -> &RangeSet<value::Real> {
		&self.bounds
	}
}

#[derive(Clone)]
pub struct RationalRestrictions {
	bounds: RangeSet<value::Rational>
}

impl RationalRestrictions {
	pub fn bounds(&self) -> &RangeSet<value::Rational> {
		&self.bounds
	}
}

#[derive(Clone)]
pub struct FloatRestrictions {
	bounds: RangeSet<NotNan<f32>>
}

impl FloatRestrictions {
	pub fn bounds(&self) -> &RangeSet<NotNan<f32>> {
		&self.bounds
	}
}

#[derive(Clone)]
pub struct DoubleRestrictions {
	bounds: RangeSet<NotNan<f64>>
}

impl DoubleRestrictions {
	pub fn bounds(&self) -> &RangeSet<NotNan<f64>> {
		&self.bounds
	}
}

/// String restrictions.
/// 
/// # Facets
/// 
/// - length (including minLength, maxLength)
/// - pattern (including enumeration)
#[derive(Clone)]
pub struct StringRestrictions {
	pattern: Option<RegExp>
}

impl StringRestrictions {
	pub fn pattern(&self) -> Option<&RegExp> {
		self.pattern.as_ref()
	}
}