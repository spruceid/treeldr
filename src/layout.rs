use crate::Id;

/// Layout definition.
pub struct Definition {
	id: Id
}

impl Definition {
	/// Returns the identifier of the defined layout.
	pub fn id(&self) -> Id {
		self.id
	}
}