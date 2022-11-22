use crate::{metadata::Merge, vocab};
use locspan::{MapLocErr, Meta};
use locspan_derive::{StrippedEq, StrippedPartialEq};

pub mod cardinal;

/// Container restriction.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Restriction {
	Cardinal(cardinal::Restriction),
}

impl Restriction {
	pub fn as_binding(&self) -> Binding {
		match self {
			Self::Cardinal(r) => Binding::Cardinal(r.as_binding())
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

pub enum Binding {
	Cardinal(cardinal::Binding)
}

impl Binding {
	pub fn property(&self) -> Property {
		match self {
			Self::Cardinal(b) => b.property()
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Property {
	MinCardinality,
	MaxCardinality,
	InclusiveMinimum,
	ExclusiveMinimum,
	InclusiveMaximum,
	ExclusiveMaximum,
	MinLength,
	MaxLength,
	Pattern
}

impl Property {
	pub fn term(&self) -> vocab::Term {
		use vocab::{Term, TreeLdr};
		match self {
			Self::MinCardinality => Term::TreeLdr(TreeLdr::MinCardinality),
			Self::MaxCardinality => Term::TreeLdr(TreeLdr::MaxCardinality),
			Self::InclusiveMinimum => Term::TreeLdr(TreeLdr::InclusiveMinimum),
			Self::ExclusiveMinimum => Term::TreeLdr(TreeLdr::ExclusiveMinimum),
			Self::InclusiveMaximum => Term::TreeLdr(TreeLdr::InclusiveMaximum),
			Self::ExclusiveMaximum => Term::TreeLdr(TreeLdr::ExclusiveMaximum),
			Self::MinLength => Term::TreeLdr(TreeLdr::MinLength),
			Self::MaxLength => Term::TreeLdr(TreeLdr::MaxLength),
			Self::Pattern => Term::TreeLdr(TreeLdr::Pattern),
		}
	}

	pub fn name(&self) -> &'static str {
		match self {
			Self::MinCardinality => "minimum cardinality",
			Self::MaxCardinality => "maximum cardinality",
			Self::InclusiveMinimum => "inclusive minimum",
			Self::ExclusiveMinimum => "exclusive minimum",
			Self::InclusiveMaximum => "inclusive maximum",
			Self::ExclusiveMaximum => "exclusive maximum",
			Self::MinLength => "minimum length",
			Self::MaxLength => "maximum length",
			Self::Pattern => "pattern",
		}
	}
}