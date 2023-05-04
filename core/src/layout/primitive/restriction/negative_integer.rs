use xsd_types::NegativeInteger;

use crate::layout::Primitive;

use super::{template, RestrainableType};

pub type Restriction = template::integer::Restriction<NegativeInteger>;

pub type RestrictionRef<'a> = template::integer::RestrictionRef<'a, NegativeInteger>;

pub type Restrictions<M> = template::integer::Restrictions<NegativeInteger, M>;

pub type Conflict<M> = template::integer::Conflict<NegativeInteger, M>;

pub type Iter<'a, M> = template::integer::Iter<'a, NegativeInteger, M>;

impl RestrainableType for xsd_types::NegativeInteger {
	const PRIMITIVE: Primitive = Primitive::NegativeInteger;

	type RestrictionRef<'a> = RestrictionRef<'a>;
	type Restrictions<M> = Restrictions<M>;
	type RestrictionsIter<'a, M> = Iter<'a, M> where M: 'a;
}
