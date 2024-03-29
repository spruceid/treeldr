use std::collections::BTreeMap;

use crate::{Dataset, Pattern};

use crate::abs::RegExp;

pub use crate::layout::literal::id::IdLayoutType;

pub struct IdLayout<R> {
	pub input: u32,

	pub intro: u32,

	pub dataset: Dataset<R>,

	pub pattern: Option<RegExp>,

	pub resource: Pattern<R>,

	/// Additional properties.
	pub properties: BTreeMap<R, R>,
}

impl<R: Clone> IdLayout<R> {
	pub fn build(&self) -> crate::layout::IdLayout<R> {
		crate::layout::IdLayout {
			input: self.input,
			intro: self.intro,
			dataset: self.dataset.clone(),
			pattern: self.pattern.as_ref().map(RegExp::build),
			resource: self.resource.clone(),
			extra_properties: self.properties.clone(),
		}
	}
}
