mod intersection;
pub mod list;
mod literal;
pub mod product;
pub mod sum;
mod r#union;

use crate::{layout::LayoutType, Ref};
pub use list::{
	ListLayout, ListLayoutType, OrderedListLayout, SizedListLayout, UnorderedListLayout,
};
pub use literal::{
	BooleanLayout, ByteStringLayout, DataLayout, IdLayout, IdLayoutType, LiteralLayout,
	LiteralLayoutType, NumberLayout, TextStringLayout, UnitLayout,
};
pub use product::ProductLayout;
pub use sum::SumLayout;

/// Layout.
pub enum Layout<R> {
	/// Matches nothing.
	Never,

	/// Matches literal values.
	Literal(LiteralLayout<R>),

	/// Matches objects/records.
	Product(ProductLayout<R>),

	/// Matches lists.
	List(ListLayout<R>),

	/// Matches exactly one of the given layouts.
	Sum(SumLayout<R>),

	/// Matches anything.
	Always,

	/// Layout union.
	Union(Vec<Ref<LayoutType, R>>),

	/// Layout intersection.
	Intersection(Vec<Ref<LayoutType, R>>),
}

impl<R: Clone> Layout<R> {
	pub fn build(&self) -> crate::Layout<R> {
		match self {
			Self::Never => crate::Layout::Never,
			Self::Literal(layout) => crate::Layout::Literal(layout.build()),
			Self::Product(layout) => crate::Layout::Product(layout.clone()),
			Self::List(layout) => crate::Layout::List(layout.clone()),
			Self::Sum(layout) => crate::Layout::Sum(layout.clone()),
			Self::Always => crate::Layout::Always,
			Self::Union(_layout) => {
				todo!()
			}
			Self::Intersection(_layout) => {
				todo!()
			}
		}
	}
}
