use educe::Educe;
use std::collections::BTreeMap;
use std::hash::Hash;

use crate::{Dataset, ValueFormat};

pub struct ProductLayoutType;

#[derive(Debug, Clone, Educe, serde::Serialize, serde::Deserialize)]
#[educe(
	PartialEq(bound = "R: Ord"),
	Eq(bound = "R: Ord"),
	Ord(bound = "R: Ord"),
	Hash(bound = "R: Ord + Hash")
)]
#[serde(bound(deserialize = "R: Ord + serde::Deserialize<'de>"))]
pub struct ProductLayout<R> {
	/// Number of inputs.
	pub input: u32,

	/// Number of introduced variables.
	pub intro: u32,

	/// Fields.
	pub fields: BTreeMap<String, Field<R>>,

	/// Dataset.
	pub dataset: Dataset<R>,
}

impl<R: Ord> PartialOrd for ProductLayout<R> {
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
pub struct Field<R> {
	/// Intros.
	pub intro: u32,

	/// Format.
	pub value: ValueFormat<R>,

	/// Dataset.
	pub dataset: Dataset<R>,
}

impl<R: Ord> PartialOrd for Field<R> {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}