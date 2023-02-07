use locspan::Meta;

use crate::{Multiple, MutableModel, RequiredFunctionalPropertyValue, TId, Type};

/// Union type built from `owl:unionOf`.
#[derive(Debug)]
pub struct Union<M> {
	/// Union terms.
	union_of: RequiredFunctionalPropertyValue<Multiple<TId<Type>, M>, M>,
}

impl<M> Union<M> {
	pub fn new(union_of: RequiredFunctionalPropertyValue<Multiple<TId<Type>, M>, M>) -> Self {
		Self { union_of }
	}

	pub fn union_of(&self) -> &RequiredFunctionalPropertyValue<Multiple<TId<Type>, M>, M> {
		&self.union_of
	}

	pub fn options(&self) -> &Multiple<TId<Type>, M> {
		self.union_of.value()
	}

	pub fn is_datatype(&self, model: &MutableModel<M>) -> bool {
		self.options()
			.iter()
			.all(|Meta(ty_ref, _)| model.get(*ty_ref).unwrap().as_type().is_datatype(model))
	}
}
