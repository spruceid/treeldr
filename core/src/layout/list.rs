use crate::{
	Id,
	Ref,
	WithCauses,
	prop
};

/// List layout `[T; first; rest; nil]`.
pub struct List<F> {
	/// Item layout.
	item: WithCauses<Ref<super::Definition<F>>, F>,

	/// Semantics of the list layout.
	/// 
	/// Is `None` if and only if the layout is an orphan layout.
	semantics: Option<Semantics<F>>
}

impl<F> List<F> {
	/// Creates a new list layout.
	pub fn new(
		item: WithCauses<Ref<super::Definition<F>>, F>,
		semantics: Option<Semantics<F>>
	) -> Self {
		Self {
			item,
			semantics
		}
	}

	pub fn item(&self) -> Ref<super::Definition<F>> {
		*self.item
	}

	pub fn semantics(&self) -> Option<&Semantics<F>> {
		self.semantics.as_ref()
	}
}

/// List layout semantics.
pub struct Semantics<F> {
	/// Property used to define the first item of a list node.
	first: WithCauses<Ref<prop::Definition<F>>, F>,

	/// Property used to define the rest of the list.
	rest: WithCauses<Ref<prop::Definition<F>>, F>,

	/// Value used as the empty list.
	nil: WithCauses<Id, F>,
}

impl<F> Semantics<F> {
	pub fn new(
		first: WithCauses<Ref<prop::Definition<F>>, F>,
		rest: WithCauses<Ref<prop::Definition<F>>, F>,
		nil: WithCauses<Id, F>
	) -> Self {
		Self {
			first,
			rest,
			nil
		}
	}

	pub fn first(&self) -> Ref<prop::Definition<F>> {
		*self.first
	}

	pub fn rest(&self) -> Ref<prop::Definition<F>> {
		*self.rest
	}

	pub fn nil(&self) -> Id {
		*self.nil
	}
}