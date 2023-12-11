use crate::{layout::LayoutType, Pattern, Ref};

#[derive(
	Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct ValueFormat<R> {
	/// Layout.
	pub layout: Ref<LayoutType, R>,

	/// Layout inputs.
	pub input: Vec<Pattern<R>>,

	/// Graph in which the layout is evaluated.
	pub graph: Option<Option<Pattern<R>>>,
}

impl<R> ValueFormat<R> {
	pub fn visit_dependencies<'a>(&'a self, mut f: impl FnMut(&'a Ref<LayoutType, R>)) {
		(f)(&self.layout)
	}
}
