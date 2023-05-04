use xsd_types::Integer;

use crate::layout::Primitive;

use super::{template, RestrainableType};

pub type Restriction = template::integer::Restriction<Integer>;

pub type RestrictionRef<'a> = template::integer::RestrictionRef<'a, Integer>;

pub type Restrictions<M> = template::integer::Restrictions<Integer, M>;

pub type Conflict<M> = template::integer::Conflict<Integer, M>;

pub type Iter<'a, M> = template::integer::Iter<'a, Integer, M>;

impl RestrainableType for xsd_types::Integer {
	const PRIMITIVE: Primitive = Primitive::Integer;

	type RestrictionRef<'a> = RestrictionRef<'a>;
	type Restrictions<M> = Restrictions<M>;
	type RestrictionsIter<'a, M> = Iter<'a, M> where M: 'a;
}
