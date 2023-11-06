use crate::Graph;

use super::ItemLayout;

pub struct SizedListLayout<R> {
	pub input: u32,

	pub intro: u32,

	pub items: Vec<ItemLayout<R>>,

	pub graph: Graph<R>
}