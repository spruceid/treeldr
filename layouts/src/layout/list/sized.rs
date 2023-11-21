use educe::Educe;
use std::hash::Hash;

use crate::Dataset;

use super::ItemLayout;

#[derive(Debug, Clone, Educe, serde::Serialize, serde::Deserialize)]
#[educe(
	PartialEq(bound = "R: Ord"),
	Eq(bound = "R: Ord"),
	Ord(bound = "R: Ord"),
	Hash(bound = "R: Ord + Hash")
)]
#[serde(bound(deserialize = "R: Ord + serde::Deserialize<'de>"))]
pub struct SizedListLayout<R> {
	pub input: u32,

	pub intro: u32,

	pub items: Vec<ItemLayout<R>>,

	pub dataset: Dataset<R>,
}

impl<R: Ord> PartialOrd for SizedListLayout<R> {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}
