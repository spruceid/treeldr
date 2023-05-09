use xsd_types::Time;

use crate::layout::Primitive;

use super::{template, RestrainableType};

pub type Restriction = template::none::Restriction<Time>;

pub type RestrictionRef<'a> = template::none::RestrictionRef<'a, Time>;

pub type Restrictions<M> = template::none::Restrictions<Time, M>;

pub type Conflict<M> = template::none::Conflict<Time, M>;

pub type Iter<'a, M> = template::none::Iter<'a, Time, M>;

impl RestrainableType for xsd_types::Time {
	const PRIMITIVE: Primitive = Primitive::Time;

	type RestrictionRef<'a> = RestrictionRef<'a>;
	type Restrictions<M> = Restrictions<M>;
	type RestrictionsIter<'a, M> = Iter<'a, M> where M: 'a;
}
