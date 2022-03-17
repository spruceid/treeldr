use super::Error;
use crate::{Causes, Id, WithCauses};
use derivative::Derivative;
use locspan::Location;
use std::collections::HashMap;

/// Type definition.
#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct Definition<F> {
	/// Properties.
	properties: HashMap<Id, Causes<F>>,
}

impl<F> Definition<F> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn properties(&self) -> impl Iterator<Item = (Id, &Causes<F>)> {
		self.properties.iter().map(|(p, c)| (*p, c))
	}

	pub fn declare_property(&mut self, prop_ref: Id, cause: Option<Location<F>>)
	where
		F: Ord,
	{
		use std::collections::hash_map::Entry;
		match self.properties.entry(prop_ref) {
			Entry::Vacant(entry) => {
				entry.insert(cause.into());
			}
			Entry::Occupied(mut entry) => {
				if let Some(cause) = cause {
					entry.get_mut().add(cause)
				}
			}
		}
	}
}

impl<F: Ord + Clone> WithCauses<Definition<F>, F> {
	pub fn build(
		self,
		id: Id,
		nodes: &super::context::AllocatedNodes<F>,
	) -> Result<crate::ty::Definition<F>, Error<F>> {
		let (def, causes) = self.into_parts();
		let mut result = crate::ty::Definition::new(id, causes);

		for (prop_id, prop_causes) in def.properties {
			let prop_ref = nodes.require_property(prop_id, prop_causes.preferred().cloned())?;
			result.insert_property(*prop_ref.inner(), prop_causes)
		}

		Ok(result)
	}
}
