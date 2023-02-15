use crate::{PropertyValues, TId, Type};
use derivative::Derivative;

/// Normal type.
#[derive(Debug, Derivative)]
pub struct Normal<M> {
	/// RDF Syntax `subClassOf` property.
	///
	/// Only direct super classes are listed.
	sub_class_of: PropertyValues<TId<Type>, M>,
}

impl<M> Normal<M> {
	/// Create a new normal type.
	///
	/// The `sub_class_of` values should contain all and only the direct super classes of this type,
	/// excluding unions and intersections.
	pub fn new(sub_class_of: PropertyValues<TId<Type>, M>) -> Self {
		Self { sub_class_of }
	}

	pub fn sub_class_of(&self) -> &PropertyValues<TId<Type>, M> {
		&self.sub_class_of
	}
}
