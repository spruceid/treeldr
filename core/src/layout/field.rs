use crate::vocab;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Property {
	For,
}

impl Property {
	pub fn term(&self) -> vocab::Term {
		use vocab::{Term, TreeLdr};
		match self {
			Self::For => Term::TreeLdr(TreeLdr::FieldFor),
		}
	}

	pub fn name(&self) -> &'static str {
		match self {
			Self::For => "field property",
		}
	}
}