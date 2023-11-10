use std::collections::BTreeMap;

use crate::{Dataset, ValueFormat};

pub struct ProductLayoutType;

#[derive(Debug, Clone)]
pub struct ProductLayout<R> {
	/// Number of inputs.
	pub input: u32,

	/// Number of introduced variables.
	pub intro: u32,

	/// Fields.
	pub fields: BTreeMap<String, Field<R>>,

	/// Dataset.
	pub dataset: Dataset<R>,
}

#[derive(Debug, Clone)]
pub struct Field<R> {
	/// Intros.
	pub intro: u32,

	/// Format.
	pub value: ValueFormat<R>,

	/// Dataset.
	pub dataset: Dataset<R>,
}
