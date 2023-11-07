pub mod ordered;
pub mod sized;
pub mod unordered;

pub use ordered::OrderedListLayout;
pub use sized::SizedListLayout;
pub use unordered::UnorderedListLayout;

use crate::{graph::Dataset, Format};

pub struct ListLayoutType;

pub enum ListLayout<R> {
	Unordered(UnorderedListLayout<R>),
	Ordered(OrderedListLayout<R>),
	Sized(SizedListLayout<R>),
}

pub struct ItemLayout<R> {
	/// Intros.
	pub intro: u32,

	/// Format.
	pub format: Format<R>,

	/// Dataset.
	pub dataset: Dataset<R>,
}
