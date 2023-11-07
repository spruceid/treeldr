use crate::{layout::LayoutType, Pattern, Ref};

#[derive(Clone)]
pub struct Format<R> {
	/// Layout.
	pub layout: Ref<LayoutType, R>,

	/// Layout inputs.
	pub inputs: Vec<Pattern<R>>,

	/// Graph in which the layout is evaluated.
	pub graph: Option<Option<Pattern<R>>>,
}