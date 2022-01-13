use crate::Id;

/// Type definition.
pub struct Definition {
	id: Id
}

impl Definition {
	/// Returns the identifier of the defined type.
	pub fn id(&self) -> Id {
		self.id
	}
}