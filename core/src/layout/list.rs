pub mod ordered;
pub mod sized;
pub mod unordered;

pub use ordered::OrderedListLayout;
pub use sized::SizedListLayout;
pub use unordered::UnorderedListLayout;

use crate::{graph::Dataset, Format};

pub struct ListLayoutType;

#[derive(Clone)]
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

#[derive(Clone)]
pub struct ItemLayout<R> {
	/// Intros.
	pub intro: u32,

	/// Format.
	pub format: Format<R>,

	/// Dataset.
	pub dataset: Dataset<R>,
}
