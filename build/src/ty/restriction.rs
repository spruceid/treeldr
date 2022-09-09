use crate::{context, Error};
use locspan::Meta;
use std::collections::BTreeMap;
use treeldr::{Id, metadata::Merge};

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
pub struct Restriction<M> {
	property: Meta<Id, M>,
	restrictions: BTreeMap<PropertyRestriction, M>,
}

impl<M> PartialEq for Restriction<M> {
	fn eq(&self, other: &Self) -> bool {
		*self.property == *other.property
			&& self.restrictions.len() == other.restrictions.len()
			&& self
				.restrictions
				.keys()
				.zip(other.restrictions.keys())
				.all(|(a, b)| a == b)
	}
}

impl<M> Restriction<M> {
	pub fn new(property: Meta<Id, M>) -> Self {
		Self {
			property,
			restrictions: BTreeMap::new(),
		}
	}

	pub fn add_restriction(&mut self, r: PropertyRestriction, metadata: M)
	where
		M: Merge,
	{
		use std::collections::btree_map::Entry;
		match self.restrictions.entry(r) {
			Entry::Vacant(entry) => {
				entry.insert(metadata.into());
			}
			Entry::Occupied(mut entry) => {
				entry.get_mut().merge_with(metadata)
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
		let Meta(prop_id, prop_causes) = self.property;
		let prop_ref = nodes.require_property(prop_id, &prop_causes)?;

		let mut restrictions = treeldr::prop::Restrictions::new();
		for (restriction, restriction_causes) in self.restrictions {
			if let Err(treeldr::prop::restriction::Contradiction) =
				restrictions.restrict(restriction.build(nodes, &restriction_causes)?)
			{
				return Ok(treeldr::ty::Description::Empty);
			}
		}

		let result =
			treeldr::ty::Restriction::new(Meta::new(**prop_ref, prop_causes), restrictions);
		Ok(treeldr::ty::Description::Restriction(result))
	}
}

impl PropertyRestriction {
	pub fn build<M>(
		self,
		nodes: &context::allocated::Nodes<M>,
		causes: &M,
	) -> Result<treeldr::prop::Restriction<M>, Error<M>>
	where
		M: Clone,
	{
		match self {
			Self::Range(r) => Ok(treeldr::prop::Restriction::Range(r.build(nodes, causes)?)),
			Self::Cardinality(c) => Ok(treeldr::prop::Restriction::Cardinality(c)),
		}
	}
}

impl RangeRestriction {
	pub fn build<M>(
		self,
		nodes: &context::allocated::Nodes<M>,
		causes: &M,
	) -> Result<treeldr::prop::restriction::Range<M>, Error<M>>
	where
		M: Clone,
	{
		match self {
			Self::Any(id) => {
				let ty_ref = nodes.require_type(id, causes)?;
				Ok(treeldr::prop::restriction::Range::Any(**ty_ref))
			}
			Self::All(id) => {
				let ty_ref = nodes.require_type(id, causes)?;
				Ok(treeldr::prop::restriction::Range::All(**ty_ref))
			}
		}
	}
}
