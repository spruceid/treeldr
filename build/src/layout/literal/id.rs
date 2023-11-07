use treeldr::{Dataset, Pattern};

use crate::RegExp;

pub use treeldr::layout::literal::id::IdLayoutType;

pub struct IdLayout<R> {
	pub input: u32,

	pub intro: u32,

	pub dataset: Dataset<R>,

	pub pattern: Option<RegExp>,

	pub resource: Pattern<R>,
}

impl<R: Clone> IdLayout<R> {
	pub fn build(&self) -> treeldr::layout::IdLayout<R> {
		treeldr::layout::IdLayout {
			input: self.input,
			intro: self.intro,
			dataset: self.dataset.clone(),
			pattern: self.pattern.as_ref().map(RegExp::build),
			resource: self.resource.clone(),
		}
	}
}
