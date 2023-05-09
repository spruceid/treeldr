use crate::{MetaOption, PropertyValue};

use super::Primitive;
use locspan::Meta;

pub mod base64_bytes;
pub mod boolean;
pub mod byte;
pub mod date;
pub mod datetime;
pub mod double;
pub mod float;
pub mod hex_bytes;
pub mod int;
pub mod integer;
pub mod iri;
pub mod long;
pub mod negative_integer;
pub mod non_negative_integer;
pub mod non_positive_integer;
pub mod positive_integer;
pub mod short;
pub mod string;
pub mod template;
pub mod time;
pub mod unsigned_byte;
pub mod unsigned_int;
pub mod unsigned_long;
pub mod unsigned_short;
pub mod uri;
pub mod url;

pub trait RestrainableType {
	type RestrictionRef<'a>;

	type Restrictions<M>;

	type RestrictionsIter<'a, M>
	where
		M: 'a;

	const PRIMITIVE: Primitive;
}

macro_rules! restricted_type {
	{ $( $id:ident: $ty:ty ),* } => {
		/// Restricted primitive layout.
		#[derive(Clone, Debug)]
		pub enum Restricted<M> {
			$(
				$id ( MetaOption<<$ty as RestrainableType>::Restrictions<M>, M> ),
			)*
		}

		impl<M> Restricted<M> {
			pub fn primitive(&self) -> Primitive {
				match self {
					$(
						Self::$id(_) => <$ty as RestrainableType>::PRIMITIVE,
					)*
				}
			}

			pub fn is_restricted(&self) -> bool {
				match self {
					$(
						Self::$id(r) => r.is_some_and(|r| r.is_restricted()),
					)*
				}
			}

			pub fn restrictions(&self) -> Option<Meta<Restrictions<M>, &M>> {
				match self {
					$(
						Self::$id(r) => r.as_ref().map(|m| m.borrow().map(Restrictions::$id)),
					)*
				}
			}

			pub fn with_restrictions(&self) -> Option<WithRestrictions<M>> {
				self.restrictions().and_then(WithRestrictions::new)
			}
		}

		impl<M> From<Primitive> for Restricted<M> {
			fn from(p: Primitive) -> Self {
				match p {
					$(
						Primitive::$id => Self::$id(MetaOption::default()),
					)*
				}
			}
		}

		#[derive(Clone, Copy)]
		pub enum RestrictionRef<'a> {
			$(
				$id(<$ty as RestrainableType>::RestrictionRef<'a>),
			)*
		}

		pub enum Restrictions<'a, M: 'a> {
			$(
				$id(&'a <$ty as RestrainableType>::Restrictions<M>),
			)*
		}

		impl<'a, M> Clone for Restrictions<'a, M> {
			fn clone(&self) -> Self {
				*self
			}
		}

		impl<'a, M> Copy for Restrictions<'a, M> {}

		impl<'a, M> Restrictions<'a, M> {
			pub fn is_restricted(&self) -> bool {
				match self {
					$(
						Self::$id(r) => r.is_restricted(),
					)*
				}
			}

			pub fn iter(&self) -> RestrictionsIter<'a, M> {
				match self {
					$(
						Self::$id(r) => RestrictionsIter::$id(r.iter()),
					)*
				}
			}
		}

		pub enum RestrictionsIter<'a, M: 'a> {
			None,
			$(
				$id(<$ty as RestrainableType>::RestrictionsIter<'a, M>),
			)*
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
					$(
						Self::$id(r) => r.next().map(|r| r.map(RestrictionRef::$id)),
					)*
				}
			}
		}

		impl<'a, M> DoubleEndedIterator for RestrictionsIter<'a, M> {
			fn next_back(&mut self) -> Option<Self::Item> {
				match self {
					Self::None => None,
					$(
						Self::$id(r) => r.next_back().map(|r| r.map(RestrictionRef::$id)),
					)*
				}
			}
		}

	};
}

restricted_type! {
	Boolean: xsd_types::Boolean,
	Integer: xsd_types::Integer,
	NonPositiveInteger: xsd_types::NonPositiveInteger,
	NonNegativeInteger: xsd_types::NonNegativeInteger,
	PositiveInteger: xsd_types::PositiveInteger,
	NegativeInteger: xsd_types::NegativeInteger,
	I64: xsd_types::Long,
	I32: xsd_types::Int,
	I16: xsd_types::Short,
	I8: xsd_types::Byte,
	U64: xsd_types::UnsignedLong,
	U32: xsd_types::UnsignedInt,
	U16: xsd_types::UnsignedShort,
	U8: xsd_types::UnsignedByte,
	Float: xsd_types::Float,
	Double: xsd_types::Double,
	Base64Bytes: xsd_types::Base64Binary,
	HexBytes: xsd_types::HexBinary,
	String: xsd_types::String,
	Time: xsd_types::Time,
	Date: xsd_types::Date,
	DateTime: xsd_types::DateTime,
	Iri: iri::Iri,
	Uri: uri::Uri,
	Url: url::Url
}

/// Values of the `tldr:withRestrictions` property.
pub struct WithRestrictions<'a, M> {
	pub(crate) restrictions: Meta<Restrictions<'a, M>, &'a M>,
}

impl<'a, M> WithRestrictions<'a, M> {
	fn new(restrictions: Meta<Restrictions<'a, M>, &'a M>) -> Option<Self> {
		if restrictions.is_restricted() {
			Some(Self { restrictions })
		} else {
			None
		}
	}

	pub fn iter(&self) -> WithRestrictionsIter<'a, M> {
		WithRestrictionsIter {
			restrictions: Some(self.restrictions),
		}
	}
}

/// Iterator over the values of the `tldr:withRestrictions` property.
pub struct WithRestrictionsIter<'a, M> {
	restrictions: Option<Meta<Restrictions<'a, M>, &'a M>>,
}

impl<'a, M> Iterator for WithRestrictionsIter<'a, M> {
	type Item = PropertyValue<Restrictions<'a, M>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.restrictions
			.take()
			.map(|r| PropertyValue::new(None, r))
	}
}
