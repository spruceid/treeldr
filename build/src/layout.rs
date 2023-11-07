mod intersection;
mod list;
mod literal;
mod product;
pub mod sum;
mod r#union;

pub use list::{ListLayout, ListLayoutType};
pub use literal::{
	BooleanLayout, ByteStringLayout, DataLayout, IdLayout, IdLayoutType, LiteralLayout,
	LiteralLayoutType, NumberLayout, TextStringLayout, UnitLayout,
};
pub use product::ProductLayout;
pub use sum::SumLayout;
use treeldr::{layout::LayoutType, Ref};

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
	pub fn build(&self) -> treeldr::Layout<R> {
		match self {
			Self::Never => treeldr::Layout::Never,
			Self::Literal(layout) => treeldr::Layout::Literal(layout.build()),
			Self::Product(layout) => treeldr::Layout::Product(layout.clone()),
			Self::List(layout) => treeldr::Layout::List(layout.clone()),
			Self::Sum(layout) => treeldr::Layout::Sum(layout.clone()),
			Self::Always => treeldr::Layout::Always,
			Self::Union(_layout) => {
				todo!()
			}
			Self::Intersection(_layout) => {
				todo!()
			}
		}
	}
}
