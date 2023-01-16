use crate::{ty::restriction, Multiple, MutableModel, TId, Type};
use locspan::Meta;

/// Intersection type.
#[derive(Debug)]
pub struct Intersection<M> {
	/// Types in the intersection.
	types: Multiple<TId<Type>, M>,
}

impl<M> Intersection<M> {
	pub fn new(types: Multiple<TId<Type>, M>) -> Result<Self, restriction::Contradiction> {
		// let mut properties = Properties::all();
		// for &ty_ref in types.keys() {
		// 	properties
		// 		.intersect_with(get(ty_ref).properties().ok_or(restriction::Contradiction)?)?;
		// }

		Ok(Self { types })
	}

	pub fn types(&self) -> &Multiple<TId<Type>, M> {
		&self.types
	}

	pub fn is_datatype(&self, model: &MutableModel<M>) -> bool {
		self.types
			.iter()
			.any(|Meta(ty_ref, _)| model.get(*ty_ref).unwrap().as_type().is_datatype(model))
	}
}
