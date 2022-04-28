use crate::{vocab::Name, MaybeSet, Ref, WithCauses};
use locspan::Location;

#[derive(Clone)]
pub struct Set<F> {
	/// Layout name, if any.
	name: MaybeSet<Name, F>,

	/// Item layout.
	item: Ref<super::Definition<F>>,
}

impl<F> Set<F> {
	pub fn new(name: MaybeSet<Name, F>, item: Ref<super::Definition<F>>) -> Self {
		Self { name, item }
	}

	pub fn name(&self) -> Option<&Name> {
		self.name.value()
	}

	pub fn set_name(
		&mut self,
		new_name: Name,
		cause: Option<Location<F>>,
	) -> Option<WithCauses<Name, F>>
	where
		F: Ord,
	{
		self.name.replace(new_name, cause)
	}

	pub fn into_name(self) -> MaybeSet<Name, F> {
		self.name
	}

	pub fn item_layout(&self) -> Ref<super::Definition<F>> {
		self.item
	}

	pub fn set_item_layout(&mut self, item: Ref<super::Definition<F>>) {
		self.item = item
	}
}
