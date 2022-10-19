use super::Properties;
use crate::{metadata, Model, Ref};
use std::collections::BTreeMap;

pub struct Union<M> {
	options: BTreeMap<Ref<super::Definition<M>>, M>,

	/// Properties in the union.
	properties: Properties<M>,
}

impl<M> Union<M> {
	pub fn new<'a, G>(options: BTreeMap<Ref<super::Definition<M>>, M>, get: G) -> Self
	where
		M: 'a + Clone + metadata::Merge,
		G: 'a + Fn(Ref<super::Definition<M>>) -> &'a super::Definition<M>,
	{
		let mut properties = Properties::none();
		for &ty_ref in options.keys() {
			if let Some(ty_properties) = get(ty_ref).properties() {
				properties.unite_with(ty_properties);
			}
		}

		Self {
			options,
			properties,
		}
	}

	pub fn options(&self) -> impl '_ + DoubleEndedIterator<Item = Ref<super::Definition<M>>> {
		self.options.keys().cloned()
	}

	pub fn properties(&self) -> &Properties<M> {
		&self.properties
	}

	pub fn is_datatype(&self, model: &Model<M>) -> bool {
		self.options
			.iter()
			.all(|(ty_ref, _)| model.types().get(*ty_ref).unwrap().is_datatype(model))
	}
}
