use super::{template, RestrainableType};
use crate::layout::Primitive;

#[derive(Debug)]
pub struct Uri;

pub type Restriction = template::none::Restriction<Uri>;

pub type RestrictionRef<'a> = template::none::RestrictionRef<'a, Uri>;

pub type Restrictions<M> = template::none::Restrictions<Uri, M>;

pub type Conflict<M> = template::none::Conflict<Uri, M>;

pub type Iter<'a, M> = template::none::Iter<'a, Uri, M>;

impl RestrainableType for Uri {
	const PRIMITIVE: Primitive = Primitive::Uri;

	type RestrictionRef<'a> = RestrictionRef<'a>;
	type Restrictions<M> = Restrictions<M>;
	type RestrictionsIter<'a, M> = Iter<'a, M> where M: 'a;
}
