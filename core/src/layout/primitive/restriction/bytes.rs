use crate::layout::Primitive;

use super::{template, RestrainableType};

#[derive(Debug)]
pub struct Bytes;

pub type Restriction = template::none::Restriction<Bytes>;

pub type RestrictionRef<'a> = template::none::RestrictionRef<'a, Bytes>;

pub type Restrictions<M> = template::none::Restrictions<Bytes, M>;

pub type Conflict<M> = template::none::Conflict<Bytes, M>;

pub type Iter<'a, M> = template::none::Iter<'a, Bytes, M>;

impl RestrainableType for Bytes {
	const PRIMITIVE: Primitive = Primitive::Bytes;

	type RestrictionRef<'a> = RestrictionRef<'a>;
	type Restrictions<M> = Restrictions<M>;
	type RestrictionsIter<'a, M> = Iter<'a, M> where M: 'a;
}
