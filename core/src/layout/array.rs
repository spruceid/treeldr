use super::{ContainerRestrictions, Layout};
use crate::{node::BindingValueRef, Id, MetaOption, Property, TId};
use locspan::Meta;

#[derive(Debug, Clone)]
pub struct Array<M> {
	/// Item layout.
	item: Meta<TId<Layout>, M>,

	/// Restrictions.
	restrictions: ContainerRestrictions<M>,

	/// Semantics of the list layout.
	///
	/// Is `None` if and only if the layout is an orphan layout.
	semantics: Option<Semantics<M>>,
}

impl<M> Array<M> {
	pub fn new(
		item: Meta<TId<Layout>, M>,
		restrictions: ContainerRestrictions<M>,
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

	pub fn restrictions(&self) -> &ContainerRestrictions<M> {
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

	pub fn bindings(&self) -> Bindings<M> {
		Bindings {
			first: self.first.as_ref(),
			rest: self.rest.as_ref(),
			nil: self.nil.as_ref(),
		}
	}
}

pub enum Binding {
	ArrayListFirst(TId<Property>),
	ArrayListRest(TId<Property>),
	ArrayListNil(Id),
}

impl Binding {
	pub fn property(&self) -> super::Property {
		match self {
			Self::ArrayListFirst(_) => super::Property::ArrayListFirst,
			Self::ArrayListRest(_) => super::Property::ArrayListRest,
			Self::ArrayListNil(_) => super::Property::ArrayListNil,
		}
	}

	pub fn value<'a, M>(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::ArrayListFirst(v) => BindingValueRef::Property(*v),
			Self::ArrayListRest(v) => BindingValueRef::Property(*v),
			Self::ArrayListNil(v) => BindingValueRef::Id(*v),
		}
	}
}

pub struct Bindings<'a, M> {
	first: Option<&'a Meta<TId<Property>, M>>,
	rest: Option<&'a Meta<TId<Property>, M>>,
	nil: Option<&'a Meta<Id, M>>,
}

impl<'a, M> Default for Bindings<'a, M> {
	fn default() -> Self {
		Self {
			first: None,
			rest: None,
			nil: None,
		}
	}
}

impl<'a, M> Iterator for Bindings<'a, M> {
	type Item = Meta<Binding, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.first
			.take()
			.map(|m| m.borrow().into_cloned_value().map(Binding::ArrayListFirst))
			.or_else(|| {
				self.rest
					.take()
					.map(|m| m.borrow().into_cloned_value().map(Binding::ArrayListRest))
					.or_else(|| {
						self.nil
							.take()
							.map(|m| m.borrow().into_cloned_value().map(Binding::ArrayListNil))
					})
			})
	}
}
