use crate::{utils::DetAutomaton, Dataset, Pattern};

pub struct IdLayoutType;

#[derive(Debug)]
pub struct IdLayout<R> {
	pub input: u32,

	pub intro: u32,

	pub dataset: Dataset<R>,

	pub pattern: Option<DetAutomaton<usize>>,

	pub resource: Pattern<R>,
}
