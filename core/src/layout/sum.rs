use crate::{Format, Graph};

pub mod deserialization;
pub mod serialization;

pub struct SumLayout<R> {
	/// Number of introduced variables.
	pub intro: u32,

	/// Variants.
	pub variants: Vec<Variant<R>>,
}

pub struct Variant<R> {
	/// Variant identifier.
	pub id: R,

	/// Name.
	pub name: String,

	/// Format.
	pub format: Format<R>,

	/// Graph.
	pub graph: Graph<R>,
}
