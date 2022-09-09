use crate::{context, Error};
use derivative::Derivative;
use std::collections::HashMap;
use treeldr::{Id, metadata::Merge};

/// Normal type definition.
#[derive(Clone, Derivative)]
#[derivative(Default(bound = ""))]
pub struct Normal<M> {
	/// Properties.
	properties: HashMap<Id, M>,
}

impl<M> Normal<M> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn is_empty(&self) -> bool {
		self.properties.is_empty()
	}

	pub fn properties(&self) -> impl Iterator<Item = (Id, &M)> {
		self.properties.iter().map(|(p, c)| (*p, c))
	}

	pub fn declare_property(&mut self, prop_ref: Id, cause: M) where M: Merge {
		use std::collections::hash_map::Entry;
		match self.properties.entry(prop_ref) {
			Entry::Vacant(entry) => {
				entry.insert(cause.into());
			}
			Entry::Occupied(mut entry) => {
				entry.get_mut().merge_with(cause)
			}
		}
	}

	pub fn build(
		self,
		nodes: &context::allocated::Nodes<M>,
	) -> Result<treeldr::ty::Description<M>, Error<M>>
	where
		M: Clone + Merge,
	{
		let mut result = treeldr::ty::Normal::new();

		for (prop_id, prop_causes) in self.properties {
			let prop_ref = nodes.require_property(prop_id, &prop_causes)?;
			result.insert_property(**prop_ref, prop_causes)
		}

		Ok(treeldr::ty::Description::Normal(result))
	}
}
