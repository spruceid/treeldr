use xsd_types::Int;

use crate::layout::Primitive;

use super::{template, RestrainableType};

pub type Restriction = template::integer::Restriction<Int>;

pub type RestrictionRef<'a> = template::integer::RestrictionRef<'a, Int>;

pub type Restrictions<M> = template::integer::Restrictions<Int, M>;

pub type Conflict<M> = template::integer::Conflict<Int, M>;

pub type Iter<'a, M> = template::integer::Iter<'a, Int, M>;

impl RestrainableType for xsd_types::Int {
	const PRIMITIVE: Primitive = Primitive::I32;

	type RestrictionRef<'a> = RestrictionRef<'a>;
	type Restrictions<M> = Restrictions<M>;
	type RestrictionsIter<'a, M> = Iter<'a, M> where M: 'a;
}
