use super::Error;
use crate::{Causes, Id, WithCauses};
use derivative::Derivative;
use locspan::Location;
use std::collections::HashMap;

/// Type definition.
pub enum Definition<F> {
	/// Normal type.
	Normal(Normal<F>),

	/// Union/sum type.
	Union(Union<F>)
}

impl<F> Default for Definition<F> {
	fn default() -> Self {
		Self::Normal(Normal::default())
	}
}

impl<F> Definition<F> {
	/// Create a new type.
	/// 
	/// By default, a normal type is created.
	/// It can later be changed into a non-normal type as long as no properties
	/// have been defined on it.
	pub fn new() -> Self {
		Self::default()
	}

	/// Declare a property of type.
	/// 
	/// The type must be normal.
	pub fn declare_property(&mut self, prop_ref: Id, cause: Option<Location<F>>)
	where
		F: Ord,
	{
		match self {
			Self::Normal(n) => n.declare_property(prop_ref, cause),
			Self::Union(_) => todo!()
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
		
		let desc = match def {
			Definition::Normal(n) => n.build(nodes)?,
			Definition::Union(u) => todo!()
		};

		Ok(crate::ty::Definition::new(id, desc, causes))
	}
}

/// Normal type definition.
#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct Normal<F> {
	/// Properties.
	properties: HashMap<Id, Causes<F>>,
}

impl<F> Normal<F> {
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

	pub fn build(
		self,
		nodes: &super::context::AllocatedNodes<F>,
	) -> Result<crate::ty::Description<F>, Error<F>> where F: Clone + Ord {
		let mut result = crate::ty::Normal::new();

		for (prop_id, prop_causes) in self.properties {
			let prop_ref = nodes.require_property(prop_id, prop_causes.preferred().cloned())?;
			result.insert_property(*prop_ref.inner(), prop_causes)
		}

		Ok(crate::ty::Description::Normal(result))
	}
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct Union<F> {
	options: HashMap<Id, Causes<F>>
}