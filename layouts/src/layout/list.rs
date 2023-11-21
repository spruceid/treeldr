pub mod ordered;
pub mod sized;
pub mod unordered;

use educe::Educe;
pub use ordered::OrderedListLayout;
pub use sized::SizedListLayout;
use std::hash::Hash;
pub use unordered::UnorderedListLayout;

use crate::{graph::Dataset, ValueFormat};

pub struct ListLayoutType;

#[derive(Debug, Clone, Educe, serde::Serialize, serde::Deserialize)]
#[educe(
	PartialEq(bound = "R: Ord"),
	Eq(bound = "R: Ord"),
	Ord(bound = "R: Ord"),
	Hash(bound = "R: Ord + Hash")
)]
#[serde(bound(deserialize = "R: Ord + serde::Deserialize<'de>"))]
pub enum ListLayout<R> {
	Unordered(UnorderedListLayout<R>),
	Ordered(OrderedListLayout<R>),
	Sized(SizedListLayout<R>),
}

impl<R> ListLayout<R> {
	pub fn input_count(&self) -> u32 {
		match self {
			Self::Unordered(l) => l.input,
			Self::Ordered(l) => l.input,
			Self::Sized(l) => l.input,
		}
	}
}

impl<R: Ord> PartialOrd for ListLayout<R> {
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
pub struct ItemLayout<R> {
	/// Intros.
	pub intro: u32,

	/// Format.
	pub value: ValueFormat<R>,

	/// Dataset.
	pub dataset: Dataset<R>,
}

impl<R: Ord> PartialOrd for ItemLayout<R> {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}
