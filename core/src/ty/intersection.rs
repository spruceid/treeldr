use super::Properties;
use crate::{metadata, prop::restriction, Metadata, Model, Ref};
use std::collections::BTreeMap;

/// Intersection type.
pub struct Intersection<F> {
	/// Types in the intersection.
	types: BTreeMap<Ref<super::Definition<F>>, Metadata<F>>,

	/// Properties in the intersection.
	properties: Properties<F>,
}

impl<F> Intersection<F> {
	pub fn new<'a, G>(
		types: BTreeMap<Ref<super::Definition<F>>, Metadata<F>>,
		get: G,
	) -> Result<Self, restriction::Contradiction>
	where
		F: 'a + Clone + metadata::Merge,
		G: 'a + Fn(Ref<super::Definition<F>>) -> &'a super::Definition<F>,
	{
		let mut properties = Properties::all();
		for &ty_ref in types.keys() {
			properties
				.intersect_with(get(ty_ref).properties().ok_or(restriction::Contradiction)?)?;
		}

		Ok(Self { types, properties })
	}

	pub fn types(&self) -> impl '_ + DoubleEndedIterator<Item = Ref<super::Definition<F>>> {
		self.types.keys().cloned()
	}

	pub fn properties(&self) -> &Properties<F> {
		&self.properties
	}

	pub fn is_datatype(&self, model: &Model<F>) -> bool {
		self.types
			.iter()
			.any(|(ty_ref, _)| model.types().get(*ty_ref).unwrap().is_datatype(model))
	}
}
