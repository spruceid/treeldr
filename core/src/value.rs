use crate::Id;

mod numeric;

pub use numeric::*;

/// Value.
pub enum Value {
	Node(Id),
	Literal(Literal)
}

/// Literal value.
pub enum Literal {
	String(String),
	Numeric(Real)
}