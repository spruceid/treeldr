use crate::{ty, Causes, WithCauses, Documentation, Id};
use shelves::Ref;
use std::collections::HashMap;

/// Property definition.
pub struct Definition<F> {
	id: Id,
	domain: HashMap<Ref<ty::Definition<F>>, Causes<F>>,
	range: WithCauses<Ref<ty::Definition<F>>, F>,
	required: WithCauses<bool, F>,
	functional: WithCauses<bool, F>,
	doc: Documentation,
	causes: Causes<F>,
}

impl<F> Definition<F> {
	pub fn new(
		id: Id,
		range: WithCauses<Ref<ty::Definition<F>>, F>,
		required: WithCauses<bool, F>,
		functional: WithCauses<bool, F>,
		causes: impl Into<Causes<F>>
	) -> Self {
		Self {
			id,
			causes: causes.into(),
			domain: HashMap::new(),
			range,
			required,
			functional,
			doc: Documentation::default(),
		}
	}

	/// Returns the identifier of the defined property.
	pub fn id(&self) -> Id {
		self.id
	}

	pub fn causes(&self) -> &Causes<F> {
		&self.causes
	}

	pub fn insert_domain(&mut self, ty_ref: Ref<ty::Definition<F>>, causes: impl Into<Causes<F>>) where F: Ord {
		self.domain.insert(ty_ref, causes.into());
	}

	pub fn range(&self) -> &WithCauses<Ref<ty::Definition<F>>, F> {
		&self.range
	}

	pub fn is_required(&self) -> bool {
		*self.required.inner()
	}

	/// Checks if this property is functional,
	/// meaning that it is associated to at most one value.
	pub fn is_functional(&self) -> bool {
		*self.functional.inner()
	}

	pub fn documentation(&self) -> &Documentation {
		&self.doc
	}

	pub fn documentation_mut(&mut self) -> &mut Documentation {
		&mut self.doc
	}

	pub fn set_documentation(&mut self, doc: Documentation) {
		self.doc = doc
	}
}