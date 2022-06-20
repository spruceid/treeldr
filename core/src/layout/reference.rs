use crate::{MaybeSet, Name, Ref, WithCauses};
use locspan::Location;

/// Reference layout.
#[derive(Clone)]
pub struct Reference<F> {
	/// Optional layout name.
	name: MaybeSet<Name, F>,

	/// Layout used to store the id of the referenced resource.
	id_layout: Ref<super::Definition<F>>,
}

impl<F> Reference<F> {
	pub fn new(name: MaybeSet<Name, F>, id_layout: Ref<super::Definition<F>>) -> Self {
		Self { name, id_layout }
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

	pub fn id_layout(&self) -> Ref<super::Definition<F>> {
		self.id_layout
	}

	pub fn set_id_layout(&mut self, id_layout: Ref<super::Definition<F>>) {
		self.id_layout = id_layout
	}
}
