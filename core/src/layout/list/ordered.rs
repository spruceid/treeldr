use crate::{graph::Dataset, ValueFormat, Pattern};

#[derive(Debug, Clone)]
pub struct OrderedListLayout<R> {
	pub input: u32,

	pub intro: u32,

	/// List node layout description.
	pub node: NodeLayout<R>,

	/// Head pattern.
	pub head: Pattern<R>,

	/// Tail pattern.
	pub tail: Pattern<R>,

	pub dataset: Dataset<R>,
}

#[derive(Debug, Clone)]
pub struct NodeLayout<R> {
	pub intro: u32,

	/// Node format.
	///
	/// The layout must take one input which corresponds to the list node,
	/// and intro at least one variable corresponding to the rest of the list.
	pub value: ValueFormat<R>,

	pub dataset: Dataset<R>,
}
