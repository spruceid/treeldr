use crate::{prop, vocab::Name, Id, MaybeSet, Ref, WithCauses};
use locspan::Location;

#[derive(Clone)]
pub struct Array<F> {
	/// Layout name, if any.
	name: MaybeSet<Name, F>,

	/// Item layout.
	item: Ref<super::Definition<F>>,

	/// Semantics of the list layout.
	///
	/// Is `None` if and only if the layout is an orphan layout.
	semantics: Option<Semantics<F>>,
}

impl<F> Array<F> {
	pub fn new(
		name: MaybeSet<Name, F>,
		item: Ref<super::Definition<F>>,
		semantics: Option<Semantics<F>>,
	) -> Self {
		Self {
			name,
			item,
			semantics,
		}
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

	pub fn semantics(&self) -> Option<&Semantics<F>> {
		self.semantics.as_ref()
	}
}

/// Layout semantics.
#[derive(Clone)]
pub struct Semantics<F> {
	/// Property used to define the first item of a list node.
	first: MaybeSet<Ref<prop::Definition<F>>, F>,

	/// Property used to define the rest of the list.
	rest: MaybeSet<Ref<prop::Definition<F>>, F>,

	/// Value used as the empty list.
	nil: MaybeSet<Id, F>,
}

impl<F> Semantics<F> {
	pub fn new(
		first: MaybeSet<Ref<prop::Definition<F>>, F>,
		rest: MaybeSet<Ref<prop::Definition<F>>, F>,
		nil: MaybeSet<Id, F>,
	) -> Self {
		Self { first, rest, nil }
	}

	pub fn first(&self) -> Option<Ref<prop::Definition<F>>> {
		self.first.value().cloned()
	}

	pub fn rest(&self) -> Option<Ref<prop::Definition<F>>> {
		self.rest.value().cloned()
	}

	pub fn nil(&self) -> Option<Id> {
		self.nil.value().cloned()
	}
}
