use crate::{Dataset, Format};

pub struct SizedListLayout<R> {
	pub input: u32,

	pub intro: u32,

	pub formats: Vec<Format<R>>,

	pub dataset: Dataset<R>,
}
