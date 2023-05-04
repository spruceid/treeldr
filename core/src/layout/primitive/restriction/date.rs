use xsd_types::Date;

use crate::layout::Primitive;

use super::{template, RestrainableType};

pub type Restriction = template::none::Restriction<Date>;

pub type RestrictionRef<'a> = template::none::RestrictionRef<'a, Date>;

pub type Restrictions<M> = template::none::Restrictions<Date, M>;

pub type Conflict<M> = template::none::Conflict<Date, M>;

pub type Iter<'a, M> = template::none::Iter<'a, Date, M>;

impl RestrainableType for xsd_types::Date {
	const PRIMITIVE: Primitive = Primitive::Date;

	type RestrictionRef<'a> = RestrictionRef<'a>;
	type Restrictions<M> = Restrictions<M>;
	type RestrictionsIter<'a, M> = Iter<'a, M> where M: 'a;
}
