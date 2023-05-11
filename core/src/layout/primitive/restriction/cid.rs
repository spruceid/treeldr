use crate::layout::Primitive;

use super::{template, RestrainableType};

#[derive(Debug)]
pub struct Cid;

pub type Restriction = template::none::Restriction<Cid>;

pub type RestrictionRef<'a> = template::none::RestrictionRef<'a, Cid>;

pub type Restrictions<M> = template::none::Restrictions<Cid, M>;

pub type Conflict<M> = template::none::Conflict<Cid, M>;

pub type Iter<'a, M> = template::none::Iter<'a, Cid, M>;

impl RestrainableType for Cid {
	const PRIMITIVE: Primitive = Primitive::Cid;

	type RestrictionRef<'a> = RestrictionRef<'a>;
	type Restrictions<M> = Restrictions<M>;
	type RestrictionsIter<'a, M> = Iter<'a, M> where M: 'a;
}
