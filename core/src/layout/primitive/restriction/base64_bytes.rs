use crate::layout::Primitive;

use super::{template, RestrainableType};

pub type Restriction = template::string::Restriction;

pub type RestrictionRef<'a> = template::string::RestrictionRef<'a>;

pub type Restrictions<M> = template::string::Restrictions<M>;

pub type Conflict<M> = template::string::Conflict<M>;

pub type Iter<'a, M> = template::string::Iter<'a, M>;

impl RestrainableType for xsd_types::Base64Binary {
	const PRIMITIVE: Primitive = Primitive::Base64Bytes;

	type RestrictionRef<'a> = RestrictionRef<'a>;
	type Restrictions<M> = Restrictions<M>;
	type RestrictionsIter<'a, M> = Iter<'a, M> where M: 'a;
}
