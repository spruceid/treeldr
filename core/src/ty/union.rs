use super::Properties;
use crate::{metadata, Model, Type, TId};
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Union<M> {
	options: BTreeMap<TId<Type>, M>,

	/// Properties in the union.
	properties: Properties<M>,
}

impl<M> Union<M> {
	pub fn new<'a, G>(options: BTreeMap<TId<Type>, M>, get: G) -> Self
	where
		M: 'a + Clone + metadata::Merge,
		G: 'a + Fn(TId<Type>) -> &'a super::Definition<M>,
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

	pub fn options(&self) -> impl '_ + DoubleEndedIterator<Item = TId<Type>> {
		self.options.keys().cloned()
	}

	pub fn properties(&self) -> &Properties<M> {
		&self.properties
	}

	pub fn is_datatype(&self, model: &Model<M>) -> bool {
		self.options
			.iter()
			.all(|(ty_ref, _)| model.get(*ty_ref).unwrap().as_type().is_datatype(model))
	}
}