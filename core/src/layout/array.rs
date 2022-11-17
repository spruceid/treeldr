use super::{Restrictions, Layout};
use crate::{Property, Id, MetaOption, TId};
use locspan::Meta;

#[derive(Debug, Clone)]
pub struct Array<M> {
	/// Item layout.
	item: Meta<TId<Layout>, M>,

	/// Restrictions.
	restrictions: Restrictions<M>,

	/// Semantics of the list layout.
	///
	/// Is `None` if and only if the layout is an orphan layout.
	semantics: Option<Semantics<M>>,
}

impl<M> Array<M> {
	pub fn new(
		item: Meta<TId<Layout>, M>,
		restrictions: Restrictions<M>,
		semantics: Option<Semantics<M>>,
	) -> Self {
		Self {
			item,
			restrictions,
			semantics,
		}
	}

	pub fn item_layout(&self) -> &Meta<TId<Layout>, M> {
		&self.item
	}

	pub fn set_item_layout(&mut self, item: Meta<TId<Layout>, M>) {
		self.item = item
	}

	pub fn restrictions(&self) -> &Restrictions<M> {
		&self.restrictions
	}

	pub fn is_required(&self) -> bool {
		self.restrictions.is_required()
	}

	pub fn semantics(&self) -> Option<&Semantics<M>> {
		self.semantics.as_ref()
	}
}

/// Layout semantics.
#[derive(Debug, Clone)]
pub struct Semantics<M> {
	/// Property used to define the first item of a list node.
	first: MetaOption<TId<Property>, M>,

	/// Property used to define the rest of the list.
	rest: MetaOption<TId<Property>, M>,

	/// Value used as the empty list.
	nil: MetaOption<Id, M>,
}

impl<M> Semantics<M> {
	pub fn new(
		first: MetaOption<TId<Property>, M>,
		rest: MetaOption<TId<Property>, M>,
		nil: MetaOption<Id, M>,
	) -> Self {
		Self { first, rest, nil }
	}

	pub fn first(&self) -> Option<TId<Property>> {
		self.first.value().cloned()
	}

	pub fn rest(&self) -> Option<TId<Property>> {
		self.rest.value().cloned()
	}

	pub fn nil(&self) -> Option<Id> {
		self.nil.value().cloned()
	}
}