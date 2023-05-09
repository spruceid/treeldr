use super::{template, RestrainableType};
use crate::layout::Primitive;

#[derive(Debug)]
pub struct Url;

pub type Restriction = template::none::Restriction<Url>;

pub type RestrictionRef<'a> = template::none::RestrictionRef<'a, Url>;

pub type Restrictions<M> = template::none::Restrictions<Url, M>;

pub type Conflict<M> = template::none::Conflict<Url, M>;

pub type Iter<'a, M> = template::none::Iter<'a, Url, M>;

impl RestrainableType for Url {
	const PRIMITIVE: Primitive = Primitive::Url;

	type RestrictionRef<'a> = RestrictionRef<'a>;
	type Restrictions<M> = Restrictions<M>;
	type RestrictionsIter<'a, M> = Iter<'a, M> where M: 'a;
}
