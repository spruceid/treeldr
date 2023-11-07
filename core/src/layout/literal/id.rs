use crate::{utils::DetAutomaton, Dataset, Pattern};

pub struct IdLayoutType;

pub struct IdLayout<R> {
	pub input: u32,

	pub intro: u32,

	pub dataset: Dataset<R>,

	pub automaton: Option<DetAutomaton<usize>>,

	pub resource: Pattern<R>,
}
