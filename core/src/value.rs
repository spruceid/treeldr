use crate::{
	Id,
	vocab
};

mod numeric;

pub use numeric::*;

/// Value.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Value {
	Node(Id),
	Literal(Literal),
}

impl Value {
	pub fn is_literal(&self) -> bool {
		matches!(self, Self::Literal(_))
	}
}

/// Literal value.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Literal {
	Boolean(bool),
	Numeric(Real),
	String(String),
	Unknown(rdf_types::StringLiteral, vocab::Term)
}
