use xsd_types::Float;

use crate::layout::Primitive;

use super::{
	template::{self, float::FloatType},
	RestrainableType,
};

impl FloatType for Float {
	const INFINITY: Self = Float::INFINITY;
	const NEG_INFINITY: Self = Float::NEG_INFINITY;
}

pub type Restriction = template::float::Restriction<Float>;

pub type RestrictionRef<'a> = template::float::RestrictionRef<'a, Float>;

pub type Restrictions<M> = template::float::Restrictions<Float, M>;

pub type Conflict<M> = template::float::Conflict<Float, M>;

pub type Iter<'a, M> = template::float::Iter<'a, Float, M>;

impl RestrainableType for xsd_types::Float {
	const PRIMITIVE: Primitive = Primitive::Float;

	type RestrictionRef<'a> = RestrictionRef<'a>;
	type Restrictions<M> = Restrictions<M>;
	type RestrictionsIter<'a, M> = Iter<'a, M> where M: 'a;
}
