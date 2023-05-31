use super::{template, RestrainableType};
use crate::layout::Primitive;

#[derive(Debug)]
pub struct Iri;

pub type Restriction = template::none::Restriction<Iri>;

pub type RestrictionRef<'a> = template::none::RestrictionRef<'a, Iri>;

pub type Restrictions<M> = template::none::Restrictions<Iri, M>;

pub type Conflict<M> = template::none::Conflict<Iri, M>;

pub type Iter<'a, M> = template::none::Iter<'a, Iri, M>;

impl RestrainableType for Iri {
	const PRIMITIVE: Primitive = Primitive::Iri;

	type RestrictionRef<'a> = RestrictionRef<'a>;
	type Restrictions<M> = Restrictions<M>;
	type RestrictionsIter<'a, M> = Iter<'a, M> where M: 'a;
}
