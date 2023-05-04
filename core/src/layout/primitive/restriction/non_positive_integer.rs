use xsd_types::NonPositiveInteger;

use crate::layout::Primitive;

use super::{template, RestrainableType};

pub type Restriction = template::integer::Restriction<NonPositiveInteger>;

pub type RestrictionRef<'a> = template::integer::RestrictionRef<'a, NonPositiveInteger>;

pub type Restrictions<M> = template::integer::Restrictions<NonPositiveInteger, M>;

pub type Conflict<M> = template::integer::Conflict<NonPositiveInteger, M>;

pub type Iter<'a, M> = template::integer::Iter<'a, NonPositiveInteger, M>;

impl RestrainableType for xsd_types::NonPositiveInteger {
	const PRIMITIVE: Primitive = Primitive::NonPositiveInteger;

	type RestrictionRef<'a> = RestrictionRef<'a>;
	type Restrictions<M> = Restrictions<M>;
	type RestrictionsIter<'a, M> = Iter<'a, M> where M: 'a;
}
