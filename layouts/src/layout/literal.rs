pub mod data;
pub mod id;

pub use data::{
	BooleanLayout, BooleanLayoutType, ByteStringLayout, ByteStringLayoutType, DataLayout,
	DataLayoutType, NumberLayout, NumberLayoutType, TextStringLayout, TextStringLayoutType,
	UnitLayout, UnitLayoutType,
};
use educe::Educe;
pub use id::{IdLayout, IdLayoutType};
use std::{collections::BTreeMap, hash::Hash};

pub struct LiteralLayoutType;

#[derive(Debug, Clone, Educe, serde::Serialize, serde::Deserialize)]
#[educe(
	PartialEq(bound = "R: Ord"),
	Eq(bound = "R: Ord"),
	Ord(bound = "R: Ord"),
	Hash(bound = "R: Ord + Hash")
)]
#[serde(bound(deserialize = "R: Ord + serde::Deserialize<'de>"))]
pub enum LiteralLayout<R> {
	Data(DataLayout<R>),
	Id(IdLayout<R>),
}

impl<R> LiteralLayout<R> {
	pub fn extra_properties(&self) -> &BTreeMap<R, R> {
		match self {
			Self::Data(d) => d.extra_properties(),
			Self::Id(d) => &d.extra_properties,
		}
	}
}

impl<R: Ord> PartialOrd for LiteralLayout<R> {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}
