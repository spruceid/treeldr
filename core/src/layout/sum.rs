use crate::{Dataset, ValueFormat};

pub mod deserialization;
pub mod serialization;

pub struct SumLayoutType;

#[derive(Debug, Clone)]
pub struct SumLayout<R> {
	pub input: u32,

	/// Number of introduced variables.
	pub intro: u32,

	/// Variants.
	pub variants: Vec<Variant<R>>,

	/// Graph.
	pub dataset: Dataset<R>,
}

#[derive(Debug, Clone)]
pub struct Variant<R> {
	/// Name.
	pub name: String,

	/// Intros.
	pub intro: u32,

	/// Format.
	pub value: ValueFormat<R>,

	/// Graph.
	pub dataset: Dataset<R>,
}
