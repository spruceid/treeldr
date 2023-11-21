use educe::Educe;
use std::hash::Hash;

use crate::{graph::Dataset, Pattern, ValueFormat};

#[derive(Debug, Clone, Educe, serde::Serialize, serde::Deserialize)]
#[educe(
	PartialEq(bound = "R: Ord"),
	Eq(bound = "R: Ord"),
	Ord(bound = "R: Ord"),
	Hash(bound = "R: Ord + Hash")
)]
#[serde(bound(deserialize = "R: Ord + serde::Deserialize<'de>"))]
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

impl<R: Ord> PartialOrd for OrderedListLayout<R> {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

#[derive(Debug, Clone, Educe, serde::Serialize, serde::Deserialize)]
#[educe(
	PartialEq(bound = "R: Ord"),
	Eq(bound = "R: Ord"),
	Ord(bound = "R: Ord"),
	Hash(bound = "R: Ord + Hash")
)]
#[serde(bound(deserialize = "R: Ord + serde::Deserialize<'de>"))]
pub struct NodeLayout<R> {
	pub intro: u32,

	/// Node format.
	///
	/// The layout must take one input which corresponds to the list node,
	/// and intro at least one variable corresponding to the rest of the list.
	pub value: ValueFormat<R>,

	pub dataset: Dataset<R>,
}

impl<R: Ord> PartialOrd for NodeLayout<R> {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}
