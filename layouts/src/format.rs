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
