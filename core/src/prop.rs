use crate::{ty, BlankIdIndex, Documentation, Id, IriIndex};
use locspan::Meta;
use rdf_types::Subject;
use shelves::Ref;
use std::collections::HashMap;

pub mod restriction;

pub use restriction::{Restriction, Restrictions};

/// Property definition.
pub struct Definition<M, I = IriIndex, B = BlankIdIndex> {
	id: Subject<I, B>,
	domain: HashMap<Ref<ty::Definition<M>>, M>,
	range: Meta<Ref<ty::Definition<M>>, M>,
	required: Meta<bool, M>,
	functional: Meta<bool, M>,
	causes: M,
}

impl<M> Definition<M> {
	pub fn new(
		id: Id,
		range: Meta<Ref<ty::Definition<M>>, M>,
		required: Meta<bool, M>,
		functional: Meta<bool, M>,
		causes: impl Into<M>,
	) -> Self {
		Self {
			id,
			causes: causes.into(),
			domain: HashMap::new(),
			range,
			required,
			functional,
		}
	}

	/// Returns the identifier of the defined property.
	pub fn id(&self) -> Id {
		self.id
	}

	pub fn causes(&self) -> &M {
		&self.causes
	}

	pub fn insert_domain(&mut self, ty_ref: Ref<ty::Definition<M>>, metadata: M) {
		self.domain.insert(ty_ref, metadata);
	}

	pub fn range(&self) -> &Meta<Ref<ty::Definition<M>>, M> {
		&self.range
	}

	pub fn domain(&self) -> impl '_ + Iterator<Item = Ref<ty::Definition<M>>> {
		self.domain.keys().cloned()
	}

	pub fn is_required(&self) -> bool {
		*self.required
	}

	/// Checks if this property is functional,
	/// meaning that it is associated to at most one value.
	pub fn is_functional(&self) -> bool {
		*self.functional
	}

	pub fn label<'m>(&self, model: &'m crate::Model<M>) -> Option<&'m str> {
		model.get(self.id).unwrap().label()
	}

	pub fn documentation<'m>(&self, model: &'m crate::Model<M>) -> &'m Documentation {
		model.get(self.id).unwrap().documentation()
	}
}
