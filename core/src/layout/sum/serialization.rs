use std::hash::Hash;

use super::SumLayout;

impl<R> SumLayout<R> {
	pub fn build_serialization_tree(&self) -> Tree where R: Clone + Eq + Hash {
		todo!()
	}
}

/// Deserialization tree.
pub enum Tree {
	// Input(u32, InputTree)
}