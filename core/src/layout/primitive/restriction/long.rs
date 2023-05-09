use xsd_types::Long;

use crate::layout::Primitive;

use super::{template, RestrainableType};

pub type Restriction = template::integer::Restriction<Long>;

pub type RestrictionRef<'a> = template::integer::RestrictionRef<'a, Long>;

pub type Restrictions<M> = template::integer::Restrictions<Long, M>;

pub type Conflict<M> = template::integer::Conflict<Long, M>;

pub type Iter<'a, M> = template::integer::Iter<'a, Long, M>;

impl RestrainableType for xsd_types::Long {
	const PRIMITIVE: Primitive = Primitive::I64;

	type RestrictionRef<'a> = RestrictionRef<'a>;
	type Restrictions<M> = Restrictions<M>;
	type RestrictionsIter<'a, M> = Iter<'a, M> where M: 'a;
}
