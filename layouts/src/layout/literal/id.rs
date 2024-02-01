use educe::Educe;
use std::{collections::BTreeMap, hash::Hash};

use crate::{utils::DetAutomaton, Dataset, Pattern};

pub struct IdLayoutType;

#[derive(Debug, Clone, Educe, serde::Serialize, serde::Deserialize)]
#[educe(
	PartialEq(bound = "R: Ord"),
	Eq(bound = "R: Ord"),
	Ord(bound = "R: Ord"),
	Hash(bound = "R: Ord + Hash")
)]
#[serde(bound(deserialize = "R: Ord + serde::Deserialize<'de>"))]
pub struct IdLayout<R> {
	pub input: u32,

	pub intro: u32,

	pub dataset: Dataset<R>,

	pub pattern: Option<DetAutomaton<usize>>,

	pub resource: Pattern<R>,

	/// Additional properties.
	pub extra_properties: BTreeMap<R, R>,
}

impl<R: Ord> PartialOrd for IdLayout<R> {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}
