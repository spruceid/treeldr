use crate::Id;

/// Property definition.
pub struct Definition {
	id: Id
}

impl Definition {
	/// Returns the identifier of the defined property.
	pub fn id(&self) -> Id {
		self.id
	}
}