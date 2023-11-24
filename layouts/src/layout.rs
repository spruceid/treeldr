pub mod list;
pub mod literal;
pub mod product;
pub mod sum;

use educe::Educe;
pub use list::{
	ListLayout, ListLayoutType, OrderedListLayout, SizedListLayout, UnorderedListLayout,
};
pub use literal::{
	BooleanLayout, BooleanLayoutType, ByteStringLayout, ByteStringLayoutType, DataLayout,
	DataLayoutType, IdLayout, IdLayoutType, LiteralLayout, LiteralLayoutType, NumberLayout,
	NumberLayoutType, TextStringLayout, TextStringLayoutType, UnitLayout, UnitLayoutType,
};
pub use product::{ProductLayout, ProductLayoutType};
use std::hash::Hash;
pub use sum::{SumLayout, SumLayoutType};

use crate::{GetFromLayouts, Layouts};

/// Layout type.
pub struct LayoutType;

impl<R: Ord> GetFromLayouts<Layouts<R>, R> for LayoutType {
	type Target<'c> = &'c Layout<R> where R: 'c;

	fn get_from_layouts<'c>(
		context: &'c crate::Layouts<R>,
		r: &crate::Ref<Self, R>,
	) -> Option<Self::Target<'c>> {
		context.layout(r.id())
	}
}

/// Layout value.
#[derive(Debug, Clone, Educe, serde::Serialize, serde::Deserialize)]
#[educe(
	PartialEq(bound = "R: Ord"),
	Eq(bound = "R: Ord"),
	Ord(bound = "R: Ord"),
	Hash(bound = "R: Ord + Hash")
)]
#[serde(bound(deserialize = "R: Ord + serde::Deserialize<'de>"))]
pub enum Layout<R> {
	Never,
	Literal(LiteralLayout<R>),
	Product(ProductLayout<R>),
	List(ListLayout<R>),
	Sum(SumLayout<R>),
	Always,
}

impl<R: Ord> PartialOrd for Layout<R> {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl<R> Layout<R> {
	pub fn input_count(&self) -> Option<u32> {
		match self {
			Self::Never => None,
			Self::Literal(_) => Some(1),
			Self::Product(p) => Some(p.input),
			Self::List(l) => Some(l.input_count()),
			Self::Sum(s) => Some(s.input),
			Self::Always => None,
		}
	}
}