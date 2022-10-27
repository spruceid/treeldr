use super::Properties;
use crate::{
	metadata, prop::restriction, utils::replace_with, Id, Model, Ref, SubstituteReferences,
};
use std::collections::BTreeMap;

/// Intersection type.
pub struct Intersection<M> {
	/// Types in the intersection.
	types: BTreeMap<Ref<super::Definition<M>>, M>,

	/// Properties in the intersection.
	properties: Properties<M>,
}

impl<M> Intersection<M> {
	pub fn new<'a, G>(
		types: BTreeMap<Ref<super::Definition<M>>, M>,
		get: G,
	) -> Result<Self, restriction::Contradiction>
	where
		M: 'a + Clone + metadata::Merge,
		G: 'a + Fn(Ref<super::Definition<M>>) -> &'a super::Definition<M>,
	{
		let mut properties = Properties::all();
		for &ty_ref in types.keys() {
			properties
				.intersect_with(get(ty_ref).properties().ok_or(restriction::Contradiction)?)?;
		}

		Ok(Self { types, properties })
	}

	pub fn types(&self) -> impl '_ + DoubleEndedIterator<Item = Ref<super::Definition<M>>> {
		self.types.keys().cloned()
	}

	pub fn properties(&self) -> &Properties<M> {
		&self.properties
	}

	pub fn is_datatype(&self, model: &Model<M>) -> bool {
		self.types
			.iter()
			.any(|(ty_ref, _)| model.types().get(*ty_ref).unwrap().is_datatype(model))
	}
}

impl<M> SubstituteReferences<M> for Intersection<M> {
	fn substitute_references<I, T, P, L>(&mut self, sub: &crate::ReferenceSubstitution<I, T, P, L>)
	where
		I: Fn(Id) -> Id,
		T: Fn(Ref<super::Definition<M>>) -> Ref<super::Definition<M>>,
		P: Fn(Ref<crate::prop::Definition<M>>) -> Ref<crate::prop::Definition<M>>,
		L: Fn(Ref<crate::layout::Definition<M>>) -> Ref<crate::layout::Definition<M>>,
	{
		replace_with(&mut self.types, |v| {
			v.into_iter().map(|(r, m)| (sub.ty(r), m)).collect()
		});
		self.properties.substitute_references(sub)
	}
}
