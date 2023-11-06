use crate::{Pattern, Graph};

use super::ItemLayout;

pub struct UnsizedListLayout<R> {
	pub input: u32,

	pub intro: u32,

	pub item: ItemLayout<R>,

	pub sequence: Option<Sequence<R>>,

	pub graph: Graph<R>
}

pub struct Sequence<R> {
	/// Head pattern.
	pub head: Pattern<R>,

	/// Tail pattern. Must not be an intro variable.
	pub tail: Pattern<R>
}