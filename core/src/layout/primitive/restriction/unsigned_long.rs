use xsd_types::UnsignedLong;

use crate::layout::Primitive;

use super::{template, RestrainableType};

pub type Restriction = template::integer::Restriction<UnsignedLong>;

pub type RestrictionRef<'a> = template::integer::RestrictionRef<'a, UnsignedLong>;

pub type Restrictions<M> = template::integer::Restrictions<UnsignedLong, M>;

pub type Conflict<M> = template::integer::Conflict<UnsignedLong, M>;

pub type Iter<'a, M> = template::integer::Iter<'a, UnsignedLong, M>;

impl RestrainableType for xsd_types::UnsignedLong {
	const PRIMITIVE: Primitive = Primitive::U64;

	type RestrictionRef<'a> = RestrictionRef<'a>;
	type Restrictions<M> = Restrictions<M>;
	type RestrictionsIter<'a, M> = Iter<'a, M> where M: 'a;
}
