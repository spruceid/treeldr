use educe::Educe;
use std::hash::Hash;

use crate::{utils::DetAutomaton, Dataset, Pattern};

pub struct DataLayoutType;

pub struct UnitLayoutType;

pub struct BooleanLayoutType;

pub struct NumberLayoutType;

pub struct ByteStringLayoutType;

pub struct TextStringLayoutType;

/// Data layout.
#[derive(Debug, Clone, Educe, serde::Serialize, serde::Deserialize)]
#[educe(
	PartialEq(bound = "R: Ord"),
	Eq(bound = "R: Ord"),
	Ord(bound = "R: Ord"),
	Hash(bound = "R: Ord + Hash")
)]
#[serde(bound(deserialize = "R: Ord + serde::Deserialize<'de>"))]
pub enum DataLayout<R> {
	Unit(UnitLayout<R>),
	Boolean(BooleanLayout<R>),
	Number(NumberLayout<R>),
	ByteString(ByteStringLayout<R>),
	TextString(TextStringLayout<R>),
}

impl<R: Ord> PartialOrd for DataLayout<R> {
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
pub struct UnitLayout<R> {
	pub input: u32,

	pub intro: u32,

	pub dataset: Dataset<R>,
}

impl<R: Ord> PartialOrd for UnitLayout<R> {
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
pub struct BooleanLayout<R> {
	pub input: u32,

	pub intro: u32,

	pub dataset: Dataset<R>,

	pub resource: Pattern<R>,

	pub datatype: R,
}

impl<R: Ord> PartialOrd for BooleanLayout<R> {
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
pub struct NumberLayout<R> {
	pub input: u32,

	pub intro: u32,

	pub dataset: Dataset<R>,

	pub resource: Pattern<R>,

	pub datatype: R,
}

impl<R: Ord> PartialOrd for NumberLayout<R> {
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
pub struct ByteStringLayout<R> {
	pub input: u32,

	pub intro: u32,

	pub dataset: Dataset<R>,

	pub resource: Pattern<R>,

	pub datatype: R,
}

impl<R: Ord> PartialOrd for ByteStringLayout<R> {
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
pub struct TextStringLayout<R> {
	pub input: u32,

	pub intro: u32,

	pub pattern: Option<DetAutomaton<usize>>,

	pub dataset: Dataset<R>,

	pub resource: Pattern<R>,

	pub datatype: R,
}

impl<R: Ord> PartialOrd for TextStringLayout<R> {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}
