use crate::{
	Id,
	source::Causes
};

/// Type definition.
pub struct Definition {
	id: Id,
	causes: Causes
}

impl Definition {
	pub fn new(id: Id, causes: impl Into<Causes>) -> Self {
		Self {
			id,
			causes: causes.into()
		}
	}

	/// Returns the identifier of the defined type.
	pub fn id(&self) -> Id {
		self.id
	}

	pub fn causes(&self) -> &Causes {
		&self.causes
	}
}