use xsd_types::DateTime;

use crate::layout::Primitive;

use super::{template, RestrainableType};

pub type Restriction = template::none::Restriction<DateTime>;

pub type RestrictionRef<'a> = template::none::RestrictionRef<'a, DateTime>;

pub type Restrictions<M> = template::none::Restrictions<DateTime, M>;

pub type Conflict<M> = template::none::Conflict<DateTime, M>;

pub type Iter<'a, M> = template::none::Iter<'a, DateTime, M>;

impl RestrainableType for xsd_types::DateTime {
	const PRIMITIVE: Primitive = Primitive::DateTime;

	type RestrictionRef<'a> = RestrictionRef<'a>;
	type Restrictions<M> = Restrictions<M>;
	type RestrictionsIter<'a, M> = Iter<'a, M> where M: 'a;
}
