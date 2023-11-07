use crate::{Format, Dataset};

pub mod deserialization;
pub mod serialization;

pub struct SumLayoutType;

pub struct SumLayout<R> {
	pub input: u32,

	/// Number of introduced variables.
	pub intro: u32,

	/// Variants.
	pub variants: Vec<Variant<R>>,

	/// Graph.
	pub dataset: Dataset<R>,
}

pub struct Variant<R> {
	/// Variant identifier.
	pub id: R,

	/// Name.
	pub name: String,

	/// Intros.
	pub intro: u32,

	/// Format.
	pub format: Format<R>,

	/// Graph.
	pub dataset: Dataset<R>,
}
