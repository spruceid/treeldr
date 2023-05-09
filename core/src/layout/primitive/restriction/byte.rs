use xsd_types::Byte;

use crate::layout::Primitive;

use super::{template, RestrainableType};

pub type Restriction = template::integer::Restriction<Byte>;

pub type RestrictionRef<'a> = template::integer::RestrictionRef<'a, Byte>;

pub type Restrictions<M> = template::integer::Restrictions<Byte, M>;

pub type Conflict<M> = template::integer::Conflict<Byte, M>;

pub type Iter<'a, M> = template::integer::Iter<'a, Byte, M>;

impl RestrainableType for xsd_types::Byte {
	const PRIMITIVE: Primitive = Primitive::I8;

	type RestrictionRef<'a> = RestrictionRef<'a>;
	type Restrictions<M> = Restrictions<M>;
	type RestrictionsIter<'a, M> = Iter<'a, M> where M: 'a;
}
