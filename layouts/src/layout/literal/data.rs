use educe::Educe;
use std::{collections::BTreeMap, hash::Hash};

use crate::{utils::DetAutomaton, Dataset, Pattern, Value};

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
#[serde(bound(deserialize = "R: Clone + Ord + serde::Deserialize<'de>"))]
pub enum DataLayout<R> {
	Unit(UnitLayout<R>),
	Boolean(BooleanLayout<R>),
	Number(NumberLayout<R>),
	ByteString(ByteStringLayout<R>),
	TextString(TextStringLayout<R>),
}

impl<R> DataLayout<R> {
	pub fn extra_properties(&self) -> &BTreeMap<R, R> {
		match self {
			Self::Unit(l) => &l.extra_properties,
			Self::Boolean(l) => &l.extra_properties,
			Self::Number(l) => &l.extra_properties,
			Self::ByteString(l) => &l.extra_properties,
			Self::TextString(l) => &l.extra_properties,
		}
	}
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
#[serde(bound(deserialize = "R: Clone + Ord + serde::Deserialize<'de>"))]
pub struct UnitLayout<R> {
	pub input: u32,

	pub intro: u32,

	pub dataset: Dataset<R>,

	#[serde(rename = "const", default, skip_serializing_if = "Value::is_unit")]
	pub const_: Value,

	/// Additional properties.
	pub extra_properties: BTreeMap<R, R>,
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
#[serde(bound(deserialize = "R: Clone + Ord + serde::Deserialize<'de>"))]
pub struct BooleanLayout<R> {
	pub input: u32,

	pub intro: u32,

	pub dataset: Dataset<R>,

	pub resource: Pattern<R>,

	pub datatype: R,

	/// Additional properties.
	pub extra_properties: BTreeMap<R, R>,
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
#[serde(bound(deserialize = "R: Clone + Ord + serde::Deserialize<'de>"))]
pub struct NumberLayout<R> {
	pub input: u32,

	pub intro: u32,

	pub dataset: Dataset<R>,

	pub resource: Pattern<R>,

	pub datatype: R,

	/// Additional properties.
	pub extra_properties: BTreeMap<R, R>,
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
#[serde(bound(deserialize = "R: Clone + Ord + serde::Deserialize<'de>"))]
pub struct ByteStringLayout<R> {
	pub input: u32,

	pub intro: u32,

	pub dataset: Dataset<R>,

	pub resource: Pattern<R>,

	pub datatype: R,

	/// Additional properties.
	pub extra_properties: BTreeMap<R, R>,
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
#[serde(bound(deserialize = "R: Clone + Ord + serde::Deserialize<'de>"))]
pub struct TextStringLayout<R> {
	pub input: u32,

	pub intro: u32,

	pub pattern: Option<DetAutomaton<usize>>,

	pub dataset: Dataset<R>,

	pub resource: Pattern<R>,

	pub datatype: R,

	/// Additional properties.
	pub extra_properties: BTreeMap<R, R>,
}

impl<R: Ord> PartialOrd for TextStringLayout<R> {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}
