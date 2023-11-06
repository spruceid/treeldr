use std::collections::BTreeMap;

use crate::{Format, Graph};

pub struct ProductLayout<R> {
	/// Number of introduced variables.
	pub intro: u32,

	/// Fields.
	pub fields: BTreeMap<String, Field<R>>,
}

pub struct Field<R> {
	/// Identifier.
	pub id: R,

	/// Name.
	pub name: String,

	/// Format.
	pub format: Format<R>,

	/// Graph.
	pub graph: Graph<R>,
}