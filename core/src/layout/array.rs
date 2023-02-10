use super::{ContainerRestrictions, Layout};
use crate::{
	node::BindingValueRef, property_values, FunctionalPropertyValue, Id, MetaOption, Property, TId,
};
use derivative::Derivative;
use locspan::Meta;

#[derive(Debug, Clone)]
pub struct Array<M> {
	/// Item layout.
	item: Meta<TId<Layout>, M>,

	/// Restrictions.
	restrictions: MetaOption<ContainerRestrictions<M>, M>,

	/// Semantics of the list layout.
	///
	/// Is `None` if and only if the layout is an orphan layout.
	semantics: Option<Semantics<M>>,
}

impl<M> Array<M> {
	pub fn new(
		item: Meta<TId<Layout>, M>,
		restrictions: MetaOption<ContainerRestrictions<M>, M>,
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

	pub fn restrictions(&self) -> &MetaOption<ContainerRestrictions<M>, M> {
		&self.restrictions
	}

	pub fn is_required(&self) -> bool {
		self.restrictions
			.is_some_and(ContainerRestrictions::is_required)
	}

	pub fn semantics(&self) -> Option<&Semantics<M>> {
		self.semantics.as_ref()
	}
}

/// Layout semantics.
#[derive(Debug, Clone)]
pub struct Semantics<M> {
	/// Property used to define the first item of a list node.
	first: FunctionalPropertyValue<TId<Property>, M>,

	/// Property used to define the rest of the list.
	rest: FunctionalPropertyValue<TId<Property>, M>,

	/// Value used as the empty list.
	nil: FunctionalPropertyValue<Id, M>,
}

impl<M> Semantics<M> {
	pub fn new(
		first: FunctionalPropertyValue<TId<Property>, M>,
		rest: FunctionalPropertyValue<TId<Property>, M>,
		nil: FunctionalPropertyValue<Id, M>,
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
			first: self.first.iter(),
			rest: self.rest.iter(),
			nil: self.nil.iter(),
		}
	}
}

pub enum Binding {
	ArrayListFirst(Option<Id>, TId<Property>),
	ArrayListRest(Option<Id>, TId<Property>),
	ArrayListNil(Option<Id>, Id),
}

impl Binding {
	pub fn property(&self) -> super::Property {
		match self {
			Self::ArrayListFirst(_, _) => super::Property::ArrayListFirst,
			Self::ArrayListRest(_, _) => super::Property::ArrayListRest,
			Self::ArrayListNil(_, _) => super::Property::ArrayListNil,
		}
	}

	pub fn value<'a, M>(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::ArrayListFirst(_, v) => BindingValueRef::Property(*v),
			Self::ArrayListRest(_, v) => BindingValueRef::Property(*v),
			Self::ArrayListNil(_, v) => BindingValueRef::Id(*v),
		}
	}
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct Bindings<'a, M> {
	first: property_values::functional::Iter<'a, TId<Property>, M>,
	rest: property_values::functional::Iter<'a, TId<Property>, M>,
	nil: property_values::functional::Iter<'a, Id, M>,
}

impl<'a, M> Iterator for Bindings<'a, M> {
	type Item = Meta<Binding, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.first
			.next()
			.map(|m| m.into_cloned_class_binding(Binding::ArrayListFirst))
			.or_else(|| {
				self.rest
					.next()
					.map(|m| m.into_cloned_class_binding(Binding::ArrayListRest))
					.or_else(|| {
						self.nil
							.next()
							.map(|m| m.into_cloned_class_binding(Binding::ArrayListNil))
					})
			})
	}
}
