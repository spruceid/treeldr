use crate::layout::Primitive;

use super::{template, RestrainableType};

pub type Restriction = template::unicode_string::Restriction;

pub type RestrictionRef<'a> = template::unicode_string::RestrictionRef<'a>;

pub type Restrictions<M> = template::unicode_string::Restrictions<M>;

pub type Conflict<M> = template::unicode_string::Conflict<M>;

pub type Iter<'a, M> = template::unicode_string::Iter<'a, M>;

impl RestrainableType for xsd_types::String {
	const PRIMITIVE: Primitive = Primitive::String;

	type RestrictionRef<'a> = RestrictionRef<'a>;
	type Restrictions<M> = Restrictions<M>;
	type RestrictionsIter<'a, M> = Iter<'a, M> where M: 'a;
}
