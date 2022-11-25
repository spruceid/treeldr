use crate::vocab;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Property {
	First,
	Rest,
}

impl Property {
	pub fn term(&self) -> vocab::Term {
		use vocab::{Rdf, Term};
		match self {
			Self::First => Term::Rdf(Rdf::First),
			Self::Rest => Term::Rdf(Rdf::Rest),
		}
	}

	pub fn name(&self) -> &'static str {
		match self {
			Self::First => "first item",
			Self::Rest => "rest",
		}
	}

	pub fn expect_type(&self) -> bool {
		false
	}

	pub fn expect_layout(&self) -> bool {
		false
	}
}
