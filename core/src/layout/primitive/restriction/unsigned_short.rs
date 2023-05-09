use xsd_types::UnsignedShort;

use crate::layout::Primitive;

use super::{template, RestrainableType};

pub type Restriction = template::integer::Restriction<UnsignedShort>;

pub type RestrictionRef<'a> = template::integer::RestrictionRef<'a, UnsignedShort>;

pub type Restrictions<M> = template::integer::Restrictions<UnsignedShort, M>;

pub type Conflict<M> = template::integer::Conflict<UnsignedShort, M>;

pub type Iter<'a, M> = template::integer::Iter<'a, UnsignedShort, M>;

impl RestrainableType for xsd_types::UnsignedShort {
	const PRIMITIVE: Primitive = Primitive::U16;

	type RestrictionRef<'a> = RestrictionRef<'a>;
	type Restrictions<M> = Restrictions<M>;
	type RestrictionsIter<'a, M> = Iter<'a, M> where M: 'a;
}
