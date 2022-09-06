use crate::{context, Error};
use locspan::Location;
use std::collections::BTreeMap;
use treeldr::{Metadata, Id, WithCauses};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RangeRestriction {
	Any(Id),
	All(Id),
}

pub type CardinalityRestriction = treeldr::prop::restriction::Cardinality;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PropertyRestriction {
	Range(RangeRestriction),
	Cardinality(CardinalityRestriction),
}

#[derive(Clone)]
pub struct Restriction<F> {
	property: WithCauses<Id, F>,
	restrictions: BTreeMap<PropertyRestriction, Metadata<F>>,
}

impl<F> PartialEq for Restriction<F> {
	fn eq(&self, other: &Self) -> bool {
		self.property.inner() == other.property.inner()
			&& self.restrictions.len() == other.restrictions.len()
			&& self
				.restrictions
				.keys()
				.zip(other.restrictions.keys())
				.all(|(a, b)| a == b)
	}
}

impl<F> Restriction<F> {
	pub fn new(property: WithCauses<Id, F>) -> Self {
		Self {
			property,
			restrictions: BTreeMap::new(),
		}
	}

	pub fn add_restriction(&mut self, r: PropertyRestriction, cause: Option<Location<F>>)
	where
		F: Ord,
	{
		use std::collections::btree_map::Entry;
		match self.restrictions.entry(r) {
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
		nodes: &context::allocated::Nodes<F>,
	) -> Result<treeldr::ty::Description<F>, Error<F>>
	where
		F: Clone + Ord,
	{
		let (prop_id, prop_causes) = self.property.into_parts();
		let prop_ref = nodes.require_property(prop_id, prop_causes.preferred().cloned())?;

		let mut restrictions = treeldr::prop::Restrictions::new();
		for (restriction, restriction_causes) in self.restrictions {
			if let Err(treeldr::prop::restriction::Contradiction) =
				restrictions.restrict(restriction.build(nodes, &restriction_causes)?)
			{
				return Ok(treeldr::ty::Description::Empty);
			}
		}

		let result =
			treeldr::ty::Restriction::new(WithCauses::new(**prop_ref, prop_causes), restrictions);
		Ok(treeldr::ty::Description::Restriction(result))
	}
}

impl PropertyRestriction {
	pub fn build<F>(
		self,
		nodes: &context::allocated::Nodes<F>,
		causes: &Metadata<F>,
	) -> Result<treeldr::prop::Restriction<F>, Error<F>>
	where
		F: Clone + Ord,
	{
		match self {
			Self::Range(r) => Ok(treeldr::prop::Restriction::Range(r.build(nodes, causes)?)),
			Self::Cardinality(c) => Ok(treeldr::prop::Restriction::Cardinality(c)),
		}
	}
}

impl RangeRestriction {
	pub fn build<F>(
		self,
		nodes: &context::allocated::Nodes<F>,
		causes: &Metadata<F>,
	) -> Result<treeldr::prop::restriction::Range<F>, Error<F>>
	where
		F: Clone + Ord,
	{
		match self {
			Self::Any(id) => {
				let ty_ref = nodes.require_type(id, causes.preferred().cloned())?;
				Ok(treeldr::prop::restriction::Range::Any(**ty_ref))
			}
			Self::All(id) => {
				let ty_ref = nodes.require_type(id, causes.preferred().cloned())?;
				Ok(treeldr::prop::restriction::Range::All(**ty_ref))
			}
		}
	}
}
