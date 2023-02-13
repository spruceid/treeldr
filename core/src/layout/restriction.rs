use crate::{
	metadata::Merge,
	prop::{PropertyName, UnknownProperty},
	vocab, Id, IriIndex, TId,
};
use derivative::Derivative;
use locspan::{MapLocErr, Meta};
use locspan_derive::{StrippedEq, StrippedPartialEq};

pub mod cardinal;

#[derive(Clone, Copy)]
pub enum RestrictionRef<'a> {
	Primitive(super::primitive::RestrictionRef<'a>),
	Container(ContainerRestrictionRef<'a>),
}

/// Container restriction.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ContainerRestriction {
	Cardinal(cardinal::Restriction),
}

impl ContainerRestriction {
	pub fn as_binding_ref(&self) -> BindingRef {
		match self {
			Self::Cardinal(r) => BindingRef::Cardinal(r.as_binding_ref()),
		}
	}
}

/// Container restriction reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ContainerRestrictionRef<'a> {
	Cardinal(cardinal::RestrictionRef<'a>),
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

	pub fn is_restricted(&self) -> bool {
		self.primitive
			.as_ref()
			.map(|p| p.is_restricted())
			.unwrap_or(false)
			|| self
				.container
				.as_ref()
				.map(|c| c.is_restricted())
				.unwrap_or(false)
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
	type Item = Meta<ContainerRestrictionRef<'a>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.cardinal
			.next()
			.map(|m| m.map(ContainerRestrictionRef::Cardinal))
	}
}

impl<'a, M> DoubleEndedIterator for ContainerRestrictionsIter<'a, M> {
	fn next_back(&mut self) -> Option<Self::Item> {
		self.cardinal
			.next_back()
			.map(|m| m.map(ContainerRestrictionRef::Cardinal))
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

#[derive(Debug)]
pub enum BindingRef<'a> {
	Cardinal(cardinal::BindingRef<'a>),
}

impl<'a> BindingRef<'a> {
	pub fn property(&self) -> Property {
		match self {
			Self::Cardinal(b) => b.property(),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Property {
	MinCardinality(Option<TId<UnknownProperty>>),
	MaxCardinality(Option<TId<UnknownProperty>>),
	InclusiveMinimum(Option<TId<UnknownProperty>>),
	ExclusiveMinimum(Option<TId<UnknownProperty>>),
	InclusiveMaximum(Option<TId<UnknownProperty>>),
	ExclusiveMaximum(Option<TId<UnknownProperty>>),
	MinLength(Option<TId<UnknownProperty>>),
	MaxLength(Option<TId<UnknownProperty>>),
	Pattern(Option<TId<UnknownProperty>>),
}

impl Property {
	pub fn id(&self) -> Id {
		use vocab::{Term, TreeLdr};
		match self {
			Self::MinCardinality(None) => {
				Id::Iri(IriIndex::Iri(Term::TreeLdr(TreeLdr::MinCardinality)))
			}
			Self::MinCardinality(Some(p)) => p.id(),
			Self::MaxCardinality(None) => {
				Id::Iri(IriIndex::Iri(Term::TreeLdr(TreeLdr::MaxCardinality)))
			}
			Self::MaxCardinality(Some(p)) => p.id(),
			Self::InclusiveMinimum(None) => {
				Id::Iri(IriIndex::Iri(Term::TreeLdr(TreeLdr::InclusiveMinimum)))
			}
			Self::InclusiveMinimum(Some(p)) => p.id(),
			Self::ExclusiveMinimum(None) => {
				Id::Iri(IriIndex::Iri(Term::TreeLdr(TreeLdr::ExclusiveMinimum)))
			}
			Self::ExclusiveMinimum(Some(p)) => p.id(),
			Self::InclusiveMaximum(None) => {
				Id::Iri(IriIndex::Iri(Term::TreeLdr(TreeLdr::InclusiveMaximum)))
			}
			Self::InclusiveMaximum(Some(p)) => p.id(),
			Self::ExclusiveMaximum(None) => {
				Id::Iri(IriIndex::Iri(Term::TreeLdr(TreeLdr::ExclusiveMaximum)))
			}
			Self::ExclusiveMaximum(Some(p)) => p.id(),
			Self::MinLength(None) => Id::Iri(IriIndex::Iri(Term::TreeLdr(TreeLdr::MinLength))),
			Self::MinLength(Some(p)) => p.id(),
			Self::MaxLength(None) => Id::Iri(IriIndex::Iri(Term::TreeLdr(TreeLdr::MaxLength))),
			Self::MaxLength(Some(p)) => p.id(),
			Self::Pattern(None) => Id::Iri(IriIndex::Iri(Term::TreeLdr(TreeLdr::Pattern))),
			Self::Pattern(Some(p)) => p.id(),
		}
	}

	pub fn term(&self) -> Option<vocab::Term> {
		use vocab::{Term, TreeLdr};
		match self {
			Self::MinCardinality(None) => Some(Term::TreeLdr(TreeLdr::MinCardinality)),
			Self::MaxCardinality(None) => Some(Term::TreeLdr(TreeLdr::MaxCardinality)),
			Self::InclusiveMinimum(None) => Some(Term::TreeLdr(TreeLdr::InclusiveMinimum)),
			Self::ExclusiveMinimum(None) => Some(Term::TreeLdr(TreeLdr::ExclusiveMinimum)),
			Self::InclusiveMaximum(None) => Some(Term::TreeLdr(TreeLdr::InclusiveMaximum)),
			Self::ExclusiveMaximum(None) => Some(Term::TreeLdr(TreeLdr::ExclusiveMaximum)),
			Self::MinLength(None) => Some(Term::TreeLdr(TreeLdr::MinLength)),
			Self::MaxLength(None) => Some(Term::TreeLdr(TreeLdr::MaxLength)),
			Self::Pattern(None) => Some(Term::TreeLdr(TreeLdr::Pattern)),
			_ => None,
		}
	}

	pub fn name(&self) -> PropertyName {
		match self {
			Self::MinCardinality(None) => PropertyName::Resource("minimum cardinality"),
			Self::MinCardinality(Some(p)) => PropertyName::Other(*p),
			Self::MaxCardinality(None) => PropertyName::Resource("maximum cardinality"),
			Self::MaxCardinality(Some(p)) => PropertyName::Other(*p),
			Self::InclusiveMinimum(None) => PropertyName::Resource("inclusive minimum"),
			Self::InclusiveMinimum(Some(p)) => PropertyName::Other(*p),
			Self::ExclusiveMinimum(None) => PropertyName::Resource("exclusive minimum"),
			Self::ExclusiveMinimum(Some(p)) => PropertyName::Other(*p),
			Self::InclusiveMaximum(None) => PropertyName::Resource("inclusive maximum"),
			Self::InclusiveMaximum(Some(p)) => PropertyName::Other(*p),
			Self::ExclusiveMaximum(None) => PropertyName::Resource("exclusive maximum"),
			Self::ExclusiveMaximum(Some(p)) => PropertyName::Other(*p),
			Self::MinLength(None) => PropertyName::Resource("minimum length"),
			Self::MinLength(Some(p)) => PropertyName::Other(*p),
			Self::MaxLength(None) => PropertyName::Resource("maximum length"),
			Self::MaxLength(Some(p)) => PropertyName::Other(*p),
			Self::Pattern(None) => PropertyName::Resource("pattern"),
			Self::Pattern(Some(p)) => PropertyName::Other(*p),
		}
	}

	pub fn expect_type(&self) -> bool {
		false
	}

	pub fn expect_layout(&self) -> bool {
		false
	}
}
