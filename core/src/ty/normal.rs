use super::Properties;
use crate::{metadata, prop, Id, SubstituteReferences};
use derivative::Derivative;
use shelves::Ref;

/// Normal type.
#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct Normal<M> {
	/// Properties.
	properties: Properties<M>,
}

impl<M> Normal<M> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn properties(&self) -> &Properties<M> {
		&self.properties
	}

	/// Insert a property.
	pub fn insert_property(&mut self, prop_ref: Ref<prop::Definition<M>>, metadata: M)
	where
		M: metadata::Merge,
	{
		self.properties.insert(prop_ref, None, metadata);
	}
}

impl<M> SubstituteReferences<M> for Normal<M> {
	fn substitute_references<I, T, P, L>(&mut self, sub: &crate::ReferenceSubstitution<I, T, P, L>)
	where
		I: Fn(Id) -> Id,
		T: Fn(Ref<super::Definition<M>>) -> Ref<super::Definition<M>>,
		P: Fn(Ref<prop::Definition<M>>) -> Ref<prop::Definition<M>>,
		L: Fn(Ref<crate::layout::Definition<M>>) -> Ref<crate::layout::Definition<M>>,
	{
		self.properties.substitute_references(sub)
	}
}
