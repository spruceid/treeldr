use super::Properties;
use crate::{metadata, prop::restriction, Model, Ref};
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
