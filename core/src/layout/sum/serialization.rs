use std::collections::HashMap;

use crate::{utils::DetAutomaton, graph::Dataset};

use super::SumLayout;

impl<R> SumLayout<R> {
	pub fn serialization_discriminants(&self) -> Vec<Discriminants<R>> {
		todo!()
	}
}

pub struct Discriminants<R>(Vec<Discriminant<R>>);

pub struct Discriminant<R> {
	/// Variable bindings.
	pub bindings: HashMap<u32, Constraints>,

	/// Matching dataset.
	pub dataset: Dataset<R>
}

pub struct Constraints {
	pub iri: IriConstraints,
	pub literal: LiteralConstraints
}

pub struct IriConstraints {
	/// Automaton recognizing the IRI representation of the resource.
	pub automaton: Option<DetAutomaton<usize>>
}

pub struct LiteralConstraints {
	/// Automaton recognizing the literal representation of the resource.
	pub automaton: Option<DetAutomaton<usize>>
}