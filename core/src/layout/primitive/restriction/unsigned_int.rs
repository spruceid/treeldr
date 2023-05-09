use xsd_types::UnsignedInt;

use crate::layout::Primitive;

use super::{template, RestrainableType};

pub type Restriction = template::integer::Restriction<UnsignedInt>;

pub type RestrictionRef<'a> = template::integer::RestrictionRef<'a, UnsignedInt>;

pub type Restrictions<M> = template::integer::Restrictions<UnsignedInt, M>;

pub type Conflict<M> = template::integer::Conflict<UnsignedInt, M>;

pub type Iter<'a, M> = template::integer::Iter<'a, UnsignedInt, M>;

impl RestrainableType for xsd_types::UnsignedInt {
	const PRIMITIVE: Primitive = Primitive::U32;

	type RestrictionRef<'a> = RestrictionRef<'a>;
	type Restrictions<M> = Restrictions<M>;
	type RestrictionsIter<'a, M> = Iter<'a, M> where M: 'a;
}
