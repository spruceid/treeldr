use xsd_types::NonNegativeInteger;

use crate::layout::Primitive;

use super::{template, RestrainableType};

pub type Restriction = template::integer::Restriction<NonNegativeInteger>;

pub type RestrictionRef<'a> = template::integer::RestrictionRef<'a, NonNegativeInteger>;

pub type Restrictions<M> = template::integer::Restrictions<NonNegativeInteger, M>;

pub type Conflict<M> = template::integer::Conflict<NonNegativeInteger, M>;

pub type Iter<'a, M> = template::integer::Iter<'a, NonNegativeInteger, M>;

impl RestrainableType for xsd_types::NonNegativeInteger {
	const PRIMITIVE: Primitive = Primitive::NonNegativeInteger;

	type RestrictionRef<'a> = RestrictionRef<'a>;
	type Restrictions<M> = Restrictions<M>;
	type RestrictionsIter<'a, M> = Iter<'a, M> where M: 'a;
}
