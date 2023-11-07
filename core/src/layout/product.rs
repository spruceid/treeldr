use crate::{Dataset, Format};

pub struct ProductLayoutType;

#[derive(Clone)]
pub struct ProductLayout<R> {
	/// Number of inputs.
	pub input: u32,

	/// Number of introduced variables.
	pub intro: u32,

	/// Fields.
	pub fields: Vec<Field<R>>,

	/// Dataset.
	pub dataset: Dataset<R>,
}

#[derive(Clone)]
pub struct Field<R> {
	/// Identifier.
	pub id: R,

	/// Name.
	pub name: String,

	/// Intros.
	pub intro: u32,

	/// Format.
	pub format: Format<R>,

	/// Dataset.
	pub dataset: Dataset<R>,
}