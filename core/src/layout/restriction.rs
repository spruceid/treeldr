use crate::metadata::Merge;
use locspan::{MapLocErr, Meta};
use locspan_derive::{StrippedEq, StrippedPartialEq};

pub mod cardinal;

/// Container restriction.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Restriction {
	Cardinal(cardinal::Restriction),
}

impl Restriction {
	pub fn as_binding<'a, M>(&'a self, meta: &'a M) -> BindingRef<'a, M> {
		match self {
			Self::Cardinal(r) => BindingRef::Cardinal(r.as_binding(meta))
		}
	}
}

#[derive(Debug)]
pub enum Conflict<M> {
	Cardinal(cardinal::Conflict<M>),
}

/// Container layout restrictions.
#[derive(Clone, Debug, StrippedPartialEq, StrippedEq)]
#[locspan(ignore(M))]
pub struct Restrictions<M> {
	cardinal: cardinal::Restrictions<M>,
}

impl<M> Default for Restrictions<M> {
	fn default() -> Self {
		Self {
			cardinal: cardinal::Restrictions::default(),
		}
	}
}

impl<M> Restrictions<M> {
	pub fn is_included_in(&self, other: &Self) -> bool {
		self.cardinal.is_included_in(&other.cardinal)
	}
}

impl<M> Restrictions<M> {
	pub fn cardinal(&self) -> &cardinal::Restrictions<M> {
		&self.cardinal
	}

	pub fn cardinal_mut(&mut self) -> &mut cardinal::Restrictions<M> {
		&mut self.cardinal
	}

	pub fn is_restricted(&self) -> bool {
		self.cardinal.is_restricted()
	}

	pub fn is_required(&self) -> bool {
		self.cardinal.is_required()
	}

	pub fn insert(
		&mut self,
		Meta(restriction, meta): Meta<Restriction, M>,
	) -> Result<(), Meta<Conflict<M>, M>>
	where
		M: Clone + Merge,
	{
		match restriction {
			Restriction::Cardinal(r) => self
				.cardinal
				.insert(Meta(r, meta))
				.map_loc_err(Conflict::Cardinal),
		}
	}

	pub fn unify(&mut self, other: Self) -> Result<(), Meta<Conflict<M>, M>>
	where
		M: Clone + Merge,
	{
		self.cardinal
			.unify(other.cardinal)
			.map_loc_err(Conflict::Cardinal)
	}
}

pub enum BindingRef<'a, M> {
	Cardinal(cardinal::BindingRef<'a, M>)
}