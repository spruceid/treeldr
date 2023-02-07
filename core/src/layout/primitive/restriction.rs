use crate::{
	property_values, FunctionalPropertyValue, PropertyValue, PropertyValues,
	RequiredFunctionalPropertyValue,
};

use super::Primitive;
use derivative::Derivative;
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
	Integer(FunctionalPropertyValue<integer::Restrictions<M>, M>),
	UnsignedInteger(FunctionalPropertyValue<unsigned::Restrictions<M>, M>),
	Float(FunctionalPropertyValue<float::Restrictions<M>, M>),
	Double(FunctionalPropertyValue<double::Restrictions<M>, M>),
	String(FunctionalPropertyValue<string::Restrictions<M>, M>),
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
			Self::Integer(r) => r.is_some_and(|r| r.is_restricted()),
			Self::UnsignedInteger(r) => r.is_some_and(|r| r.is_restricted()),
			Self::Float(r) => r.is_some_and(|r| r.is_restricted()),
			Self::Double(r) => r.is_some_and(|r| r.is_restricted()),
			Self::String(r) => r.is_some_and(|r| r.is_restricted()),
			_ => false,
		}
	}

	pub fn restrictions(&self) -> Option<Restrictions<M>> {
		match self {
			Self::Integer(r) => r.value().map(Restrictions::Integer),
			Self::UnsignedInteger(r) => r.value().map(Restrictions::UnsignedInteger),
			Self::Float(r) => r.value().map(Restrictions::Float),
			Self::Double(r) => r.value().map(Restrictions::Double),
			Self::String(r) => r.value().map(Restrictions::String),
			_ => None,
		}
	}

	pub fn with_restrictions(&self) -> Option<WithRestrictions<M>> {
		match self {
			Self::Integer(r) => r
				.as_required()
				.map(|s| WithRestrictions::new(s, Restrictions::Integer)),
			Self::UnsignedInteger(r) => r
				.as_required()
				.map(|s| WithRestrictions::new(s, Restrictions::UnsignedInteger)),
			Self::Float(r) => r
				.as_required()
				.map(|s| WithRestrictions::new(s, Restrictions::Float)),
			Self::Double(r) => r
				.as_required()
				.map(|s| WithRestrictions::new(s, Restrictions::Double)),
			Self::String(r) => r
				.as_required()
				.map(|s| WithRestrictions::new(s, Restrictions::String)),
			_ => None,
		}
	}
}

/// Values of the `tldr:withRestrictions` property.
pub struct WithRestrictions<'a, M> {
	pub(crate) sub_properties: &'a PropertyValues<(), M>,
	pub(crate) restrictions: Restrictions<'a, M>,
}

impl<'a, M> WithRestrictions<'a, M> {
	fn new<T>(
		value: &'a RequiredFunctionalPropertyValue<T, M>,
		f: impl FnOnce(&'a T) -> Restrictions<'a, M>,
	) -> Self {
		Self {
			sub_properties: value.sub_properties(),
			restrictions: f(value.value()),
		}
	}

	pub fn iter(&self) -> WithRestrictionsIter<'a, M> {
		WithRestrictionsIter {
			sub_properties: self.sub_properties.iter(),
			restrictions: self.restrictions,
		}
	}
}

/// Iterator over the values of the `tldr:withRestrictions` property.
pub struct WithRestrictionsIter<'a, M> {
	sub_properties: property_values::non_functional::Iter<'a, (), M>,
	restrictions: Restrictions<'a, M>,
}

impl<'a, M> Iterator for WithRestrictionsIter<'a, M> {
	type Item = PropertyValue<Restrictions<'a, M>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.sub_properties.next().map(|s| {
			PropertyValue::new(
				s.sub_property,
				Meta(self.restrictions, s.value.into_metadata()),
			)
		})
	}
}

impl<M> From<Primitive> for Restricted<M> {
	fn from(p: Primitive) -> Self {
		match p {
			Primitive::Boolean => Self::Boolean,
			Primitive::Integer => Self::Integer(FunctionalPropertyValue::default()),
			Primitive::UnsignedInteger => Self::UnsignedInteger(FunctionalPropertyValue::default()),
			Primitive::Float => Self::Float(FunctionalPropertyValue::default()),
			Primitive::Double => Self::Double(FunctionalPropertyValue::default()),
			Primitive::String => Self::String(FunctionalPropertyValue::default()),
			Primitive::Time => Self::Time,
			Primitive::Date => Self::Date,
			Primitive::DateTime => Self::DateTime,
			Primitive::Iri => Self::Iri,
			Primitive::Uri => Self::Uri,
			Primitive::Url => Self::Url,
		}
	}
}

#[derive(Clone, Copy)]
pub enum RestrictionRef<'a> {
	Integer(integer::RestrictionRef<'a>),
	UnsignedInteger(unsigned::RestrictionRef<'a>),
	Float(float::Restriction),
	Double(double::Restriction),
	String(string::RestrictionRef<'a>),
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Copy(bound = ""))]
pub enum Restrictions<'a, M> {
	Integer(&'a integer::Restrictions<M>),
	UnsignedInteger(&'a unsigned::Restrictions<M>),
	Float(&'a float::Restrictions<M>),
	Double(&'a double::Restrictions<M>),
	String(&'a string::Restrictions<M>),
}

impl<'a, M> Restrictions<'a, M> {
	pub fn iter(&self) -> RestrictionsIter<'a, M> {
		match self {
			Self::Integer(r) => RestrictionsIter::Integer(r.iter()),
			Self::UnsignedInteger(r) => RestrictionsIter::UnsignedInteger(r.iter()),
			Self::Float(r) => RestrictionsIter::Float(r.iter()),
			Self::Double(r) => RestrictionsIter::Double(r.iter()),
			Self::String(r) => RestrictionsIter::String(r.iter()),
		}
	}
}

pub enum RestrictionsIter<'a, M> {
	None,
	Integer(integer::Iter<'a, M>),
	UnsignedInteger(unsigned::Iter<'a, M>),
	Float(float::Iter<'a, M>),
	Double(double::Iter<'a, M>),
	String(string::Iter<'a, M>),
}

impl<'a, M> Default for RestrictionsIter<'a, M> {
	fn default() -> Self {
		Self::None
	}
}

impl<'a, M> Iterator for RestrictionsIter<'a, M> {
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

impl<'a, M> DoubleEndedIterator for RestrictionsIter<'a, M> {
	fn next_back(&mut self) -> Option<Self::Item> {
		match self {
			Self::None => None,
			Self::Integer(r) => r.next_back().map(|r| r.map(RestrictionRef::Integer)),
			Self::UnsignedInteger(r) => r
				.next_back()
				.map(|r| r.map(RestrictionRef::UnsignedInteger)),
			Self::Float(r) => r.next_back().map(|r| r.map(RestrictionRef::Float)),
			Self::Double(r) => r.next_back().map(|r| r.map(RestrictionRef::Double)),
			Self::String(r) => r.next_back().map(|r| r.map(RestrictionRef::String)),
		}
	}
}
