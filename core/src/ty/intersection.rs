use crate::{ty::restriction, Multiple, MutableModel, RequiredFunctionalPropertyValue, TId, Type};
use locspan::Meta;

/// Intersection type built from `owl:intersectionOf`.
#[derive(Debug)]
pub struct Intersection<M> {
	/// Types in the intersection.
	intersection_of: RequiredFunctionalPropertyValue<Multiple<TId<Type>, M>, M>,
}

impl<M> Intersection<M> {
	pub fn new(
		intersection_of: RequiredFunctionalPropertyValue<Multiple<TId<Type>, M>, M>,
	) -> Result<Self, restriction::Contradiction> {
		Ok(Self { intersection_of })
	}

	pub fn intersection_of(&self) -> &RequiredFunctionalPropertyValue<Multiple<TId<Type>, M>, M> {
		&self.intersection_of
	}

	pub fn types(&self) -> &Multiple<TId<Type>, M> {
		self.intersection_of.value()
	}

	pub fn is_datatype(&self, model: &MutableModel<M>) -> bool {
		self.types()
			.iter()
			.any(|Meta(ty_ref, _)| model.get(*ty_ref).unwrap().as_type().is_datatype(model))
	}
}
