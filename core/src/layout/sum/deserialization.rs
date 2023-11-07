use std::collections::HashMap;

use crate::utils::DetAutomaton;

use super::SumLayout;

impl<R> SumLayout<R> {
	pub fn deserialization_discriminants(&self) -> Vec<Discriminants> {
		todo!()
	}
}

pub struct Discriminants(Vec<Discriminant>);

pub enum Discriminant {
	Literal(LiteralDiscriminant),
	Record(RecordDiscriminant),
	List(ListDiscriminant),
	Any,
}

pub struct LiteralDiscriminant {
	/// Automaton recognizing the literal value.
	pub automaton: DetAutomaton<usize>,
}

pub struct RecordDiscriminant {
	pub fields: HashMap<String, Discriminant>,
}

pub enum ListDiscriminant {
	Sized(SizedListDiscriminant),
	UnsizedListDiscriminant(UnsizedListDiscriminant),
}

pub struct SizedListDiscriminant {
	pub items: Vec<Discriminant>,
}

pub struct UnsizedListDiscriminant {
	pub item: Box<Discriminant>,
}
