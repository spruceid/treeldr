use xsd_types::Short;

use crate::layout::Primitive;

use super::{template, RestrainableType};

pub type Restriction = template::integer::Restriction<Short>;

pub type RestrictionRef<'a> = template::integer::RestrictionRef<'a, Short>;

pub type Restrictions<M> = template::integer::Restrictions<Short, M>;

pub type Conflict<M> = template::integer::Conflict<Short, M>;

pub type Iter<'a, M> = template::integer::Iter<'a, Short, M>;

impl RestrainableType for xsd_types::Short {
	const PRIMITIVE: Primitive = Primitive::I16;

	type RestrictionRef<'a> = RestrictionRef<'a>;
	type Restrictions<M> = Restrictions<M>;
	type RestrictionsIter<'a, M> = Iter<'a, M> where M: 'a;
}
