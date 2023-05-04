use xsd_types::UnsignedByte;

use crate::layout::Primitive;

use super::{template, RestrainableType};

pub type Restriction = template::integer::Restriction<UnsignedByte>;

pub type RestrictionRef<'a> = template::integer::RestrictionRef<'a, UnsignedByte>;

pub type Restrictions<M> = template::integer::Restrictions<UnsignedByte, M>;

pub type Conflict<M> = template::integer::Conflict<UnsignedByte, M>;

pub type Iter<'a, M> = template::integer::Iter<'a, UnsignedByte, M>;

impl RestrainableType for xsd_types::UnsignedByte {
	const PRIMITIVE: Primitive = Primitive::U8;

	type RestrictionRef<'a> = RestrictionRef<'a>;
	type Restrictions<M> = Restrictions<M>;
	type RestrictionsIter<'a, M> = Iter<'a, M> where M: 'a;
}
