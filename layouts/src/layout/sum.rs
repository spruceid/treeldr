use educe::Educe;
use std::hash::Hash;

use crate::{Dataset, Ref, ValueFormat};

use super::LayoutType;

pub mod deserialization;
pub mod serialization;

pub struct SumLayoutType;

#[derive(Debug, Clone, Educe, serde::Serialize, serde::Deserialize)]
#[educe(
	PartialEq(bound = "R: Ord"),
	Eq(bound = "R: Ord"),
	Ord(bound = "R: Ord"),
	Hash(bound = "R: Ord + Hash")
)]
#[serde(bound(deserialize = "R: Ord + serde::Deserialize<'de>"))]
pub struct SumLayout<R> {
	pub input: u32,

	/// Number of introduced variables.
	pub intro: u32,

	/// Variants.
	pub variants: Vec<Variant<R>>,

	/// Graph.
	pub dataset: Dataset<R>,
}

impl<R> SumLayout<R> {
	pub fn visit_dependencies<'a>(&'a self, mut f: impl FnMut(&'a Ref<LayoutType, R>)) {
		for variant in &self.variants {
			variant.value.visit_dependencies(&mut f)
		}
	}
}

impl<R: Ord> PartialOrd for SumLayout<R> {
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
pub struct Variant<R> {
	/// Name.
	pub name: String,

	/// Intros.
	pub intro: u32,

	/// Format.
	pub value: ValueFormat<R>,

	/// Graph.
	pub dataset: Dataset<R>,
}

impl<R: Ord> PartialOrd for Variant<R> {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}
