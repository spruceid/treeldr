use crate::{layout::LayoutType, Pattern, Ref};

pub struct Format<R> {
	/// Layout.
	pub layout: Ref<R, LayoutType>,

	/// Layout inputs.
	pub inputs: Vec<Pattern<R>>,

	/// Graph in which the layout is evaluated.
	pub graph: Option<Option<Pattern<R>>>,
}
