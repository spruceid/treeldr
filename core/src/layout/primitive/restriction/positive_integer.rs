use xsd_types::PositiveInteger;

use crate::layout::Primitive;

use super::{template, RestrainableType};

pub type Restriction = template::integer::Restriction<PositiveInteger>;

pub type RestrictionRef<'a> = template::integer::RestrictionRef<'a, PositiveInteger>;

pub type Restrictions<M> = template::integer::Restrictions<PositiveInteger, M>;

pub type Conflict<M> = template::integer::Conflict<PositiveInteger, M>;

pub type Iter<'a, M> = template::integer::Iter<'a, PositiveInteger, M>;

impl RestrainableType for xsd_types::PositiveInteger {
	const PRIMITIVE: Primitive = Primitive::PositiveInteger;

	type RestrictionRef<'a> = RestrictionRef<'a>;
	type Restrictions<M> = Restrictions<M>;
	type RestrictionsIter<'a, M> = Iter<'a, M> where M: 'a;
}
