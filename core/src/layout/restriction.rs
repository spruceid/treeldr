use crate::{metadata::Merge, vocab};
use derivative::Derivative;
use locspan::{MapLocErr, Meta};
use locspan_derive::{StrippedEq, StrippedPartialEq};

pub mod cardinal;

#[derive(Clone, Copy)]
pub enum RestrictionRef<'a> {
	Primitive(super::primitive::RestrictionRef<'a>),
	Container(ContainerRestriction),
}

/// Container restriction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ContainerRestriction {
	Cardinal(cardinal::Restriction),
}

impl ContainerRestriction {
	pub fn as_binding(&self) -> Binding {
		match self {
			Self::Cardinal(r) => Binding::Cardinal(r.as_binding()),
		}
	}
}

#[derive(Debug)]
pub enum Conflict<M> {
	Cardinal(cardinal::Conflict<M>),
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Copy(bound = ""))]
pub struct Restrictions<'a, M> {
	primitive: Option<super::primitive::Restrictions<'a, M>>,
	container: Option<&'a ContainerRestrictions<M>>,
}

impl<'a, M> Restrictions<'a, M> {
	pub fn new_primitive(primitive: super::primitive::Restrictions<'a, M>) -> Self {
		Self {
			primitive: Some(primitive),
			container: None,
		}
	}

	pub fn new_container(container: &'a ContainerRestrictions<M>) -> Self {
		Self {
			primitive: None,
			container: Some(container),
		}
	}

	pub fn iter(&self) -> RestrictionsIter<'a, M> {
		RestrictionsIter {
			primitive: self
				.primitive
				.as_ref()
				.map(|p| p.iter())
				.unwrap_or_default(),
			container: self
				.container
				.as_ref()
				.map(|c| c.iter())
				.unwrap_or_default(),
		}
	}
}

pub struct RestrictionsIter<'a, M> {
	primitive: super::primitive::restriction::RestrictionsIter<'a, M>,
	container: ContainerRestrictionsIter<'a, M>,
}

impl<'a, M> Iterator for RestrictionsIter<'a, M> {
	type Item = Meta<RestrictionRef<'a>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.primitive
			.next()
			.map(|r| r.map(RestrictionRef::Primitive))
			.or_else(|| {
				self.container
					.next()
					.map(|r| r.map(RestrictionRef::Container))
			})
	}
}

impl<'a, M> DoubleEndedIterator for RestrictionsIter<'a, M> {
	fn next_back(&mut self) -> Option<Self::Item> {
		self.container
			.next_back()
			.map(|r| r.map(RestrictionRef::Container))
			.or_else(|| {
				self.primitive
					.next_back()
					.map(|r| r.map(RestrictionRef::Primitive))
			})
	}
}

/// Container layout restrictions.
#[derive(Clone, Debug, StrippedPartialEq, StrippedEq)]
#[locspan(ignore(M))]
pub struct ContainerRestrictions<M> {
	cardinal: cardinal::Restrictions<M>,
}

impl<M> Default for ContainerRestrictions<M> {
	fn default() -> Self {
		Self {
			cardinal: cardinal::Restrictions::default(),
		}
	}
}

impl<M> ContainerRestrictions<M> {
	pub fn is_empty(&self) -> bool {
		!self.cardinal.is_required()
	}

	pub fn as_restricted(&self) -> Option<&Self> {
		if self.is_required() {
			Some(self)
		} else {
			None
		}
	}

	pub fn is_included_in(&self, other: &Self) -> bool {
		self.cardinal.is_included_in(&other.cardinal)
	}

	#[allow(clippy::should_implement_trait)]
	pub fn into_iter(self) -> impl DoubleEndedIterator<Item = Meta<ContainerRestriction, M>> {
		self.cardinal
			.into_iter()
			.map(|m| m.map(ContainerRestriction::Cardinal))
	}
}

impl<M> ContainerRestrictions<M> {
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
		Meta(restriction, meta): Meta<ContainerRestriction, M>,
	) -> Result<(), Meta<Conflict<M>, M>>
	where
		M: Clone + Merge,
	{
		match restriction {
			ContainerRestriction::Cardinal(r) => self
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

	pub fn iter(&self) -> ContainerRestrictionsIter<M> {
		ContainerRestrictionsIter {
			cardinal: self.cardinal.iter(),
		}
	}
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct ContainerRestrictionsIter<'a, M> {
	cardinal: cardinal::RestrictionsIter<'a, M>,
}

impl<'a, M> Iterator for ContainerRestrictionsIter<'a, M> {
	type Item = Meta<ContainerRestriction, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.cardinal
			.next()
			.map(|m| m.map(ContainerRestriction::Cardinal))
	}
}

impl<'a, M> DoubleEndedIterator for ContainerRestrictionsIter<'a, M> {
	fn next_back(&mut self) -> Option<Self::Item> {
		self.cardinal
			.next_back()
			.map(|m| m.map(ContainerRestriction::Cardinal))
	}
}

pub enum Binding {
	Cardinal(cardinal::Binding),
}

impl Binding {
	pub fn property(&self) -> Property {
		match self {
			Self::Cardinal(b) => b.property(),
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
	Pattern,
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

	pub fn expect_type(&self) -> bool {
		false
	}

	pub fn expect_layout(&self) -> bool {
		false
	}
}
