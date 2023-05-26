use xsd_types::Double;

use crate::layout::Primitive;

use super::{
	template::{self, float::FloatType},
	RestrainableType,
};

impl FloatType for Double {
	const INFINITY: Self = Double::INFINITY;
	const NEG_INFINITY: Self = Double::NEG_INFINITY;
}

pub type Restriction = template::float::Restriction<Double>;

pub type RestrictionRef<'a> = template::float::RestrictionRef<'a, Double>;

pub type Restrictions<M> = template::float::Restrictions<Double, M>;

pub type Conflict<M> = template::float::Conflict<Double, M>;

pub type Iter<'a, M> = template::float::Iter<'a, Double, M>;

impl RestrainableType for xsd_types::Double {
	const PRIMITIVE: Primitive = Primitive::Double;

	type RestrictionRef<'a> = RestrictionRef<'a>;
	type Restrictions<M> = Restrictions<M>;
	type RestrictionsIter<'a, M> = Iter<'a, M> where M: 'a;
}
