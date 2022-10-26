use super::container::Restrictions;
use crate::{prop, Id, MetaOption, Name, Ref};
use locspan::Meta;

#[derive(Clone)]
pub struct Array<M> {
	/// Layout name, if any.
	name: MetaOption<Name, M>,

	/// Item layout.
	item: Ref<super::Definition<M>>,

	/// Restrictions.
	restrictions: Restrictions<M>,

	/// Semantics of the list layout.
	///
	/// Is `None` if and only if the layout is an orphan layout.
	semantics: Option<Semantics<M>>,
}

impl<M> Array<M> {
	pub fn new(
		name: MetaOption<Name, M>,
		item: Ref<super::Definition<M>>,
		restrictions: Restrictions<M>,
		semantics: Option<Semantics<M>>,
	) -> Self {
		Self {
			name,
			item,
			restrictions,
			semantics,
		}
	}

	pub fn name(&self) -> Option<&Meta<Name, M>> {
		self.name.as_ref()
	}

	pub fn set_name(&mut self, new_name: Name, metadata: M) -> Option<Meta<Name, M>> {
		self.name.replace(new_name, metadata)
	}

	pub fn into_name(self) -> MetaOption<Name, M> {
		self.name
	}

	pub fn item_layout(&self) -> Ref<super::Definition<M>> {
		self.item
	}

	pub fn set_item_layout(&mut self, item: Ref<super::Definition<M>>) {
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
#[derive(Clone)]
pub struct Semantics<M> {
	/// Property used to define the first item of a list node.
	first: MetaOption<Ref<prop::Definition<M>>, M>,

	/// Property used to define the rest of the list.
	rest: MetaOption<Ref<prop::Definition<M>>, M>,

	/// Value used as the empty list.
	nil: MetaOption<Id, M>,
}

impl<M> Semantics<M> {
	pub fn new(
		first: MetaOption<Ref<prop::Definition<M>>, M>,
		rest: MetaOption<Ref<prop::Definition<M>>, M>,
		nil: MetaOption<Id, M>,
	) -> Self {
		Self { first, rest, nil }
	}

	pub fn first(&self) -> Option<Ref<prop::Definition<M>>> {
		self.first.value().cloned()
	}

	pub fn rest(&self) -> Option<Ref<prop::Definition<M>>> {
		self.rest.value().cloned()
	}

	pub fn nil(&self) -> Option<Id> {
		self.nil.value().cloned()
	}
}
