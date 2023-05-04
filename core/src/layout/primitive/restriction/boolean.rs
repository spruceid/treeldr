use xsd_types::Boolean;

use crate::layout::Primitive;

use super::{template, RestrainableType};

pub type Restriction = template::none::Restriction<Boolean>;

pub type RestrictionRef<'a> = template::none::RestrictionRef<'a, Boolean>;

pub type Restrictions<M> = template::none::Restrictions<Boolean, M>;

pub type Conflict<M> = template::none::Conflict<Boolean, M>;

pub type Iter<'a, M> = template::none::Iter<'a, Boolean, M>;

impl RestrainableType for xsd_types::Boolean {
	const PRIMITIVE: Primitive = Primitive::Boolean;

	type RestrictionRef<'a> = RestrictionRef<'a>;
	type Restrictions<M> = Restrictions<M>;
	type RestrictionsIter<'a, M> = Iter<'a, M> where M: 'a;
}
