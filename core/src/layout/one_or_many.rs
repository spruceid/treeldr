use super::container::Restrictions;
use crate::{Id, MetaOption, Name, Ref, SubstituteReferences};
use locspan::Meta;

/// "One or many" layout.
#[derive(Clone)]
pub struct OneOrMany<M> {
	/// Layout name, if any.
	name: MetaOption<Name, M>,

	/// Item layout.
	item: Ref<super::Definition<M>>,

	/// Restrictions.
	restrictions: Restrictions<M>,
}

impl<M> OneOrMany<M> {
	pub fn new(
		name: MetaOption<Name, M>,
		item: Ref<super::Definition<M>>,
		restrictions: Restrictions<M>,
	) -> Self {
		Self {
			name,
			item,
			restrictions,
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
}

impl<M> SubstituteReferences<M> for OneOrMany<M> {
	fn substitute_references<I, T, P, L>(&mut self, sub: &crate::ReferenceSubstitution<I, T, P, L>)
	where
		I: Fn(Id) -> Id,
		T: Fn(Ref<crate::ty::Definition<M>>) -> Ref<crate::ty::Definition<M>>,
		P: Fn(Ref<crate::prop::Definition<M>>) -> Ref<crate::prop::Definition<M>>,
		L: Fn(Ref<super::Definition<M>>) -> Ref<super::Definition<M>>,
	{
		self.item = sub.layout(self.item)
	}
}