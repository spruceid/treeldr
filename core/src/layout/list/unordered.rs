use crate::Dataset;

use super::ItemLayout;

#[derive(Debug, Clone)]
pub struct UnorderedListLayout<R> {
	pub input: u32,

	pub intro: u32,

	pub item: ItemLayout<R>,

	pub dataset: Dataset<R>,
}
