pub mod format;
pub mod graph;
pub mod layout;
pub mod matching;
pub mod pattern;
pub mod r#ref;
pub mod utils;
pub mod value;

use std::collections::BTreeMap;

pub use format::Format;
pub use graph::{Dataset, Graph};
pub use layout::Layout;
use layout::LayoutType;
pub use matching::Matching;
pub use pattern::Pattern;
pub use r#ref::Ref;
pub use value::{Literal, TypedLiteral, TypedValue, Value};

pub trait GetFromLayouts<C, R>: Sized {
	type Target<'c>
	where
		C: 'c,
		R: 'c;

	fn get_from_layouts<'c>(context: &'c C, r: &Ref<Self, R>) -> Option<Self::Target<'c>>;
}

/// Layout collection.
#[derive(Debug)]
pub struct Layouts<R> {
	layouts: BTreeMap<R, Layout<R>>,
}

impl<R> Layouts<R> {
	pub fn new() -> Self {
		Self {
			layouts: BTreeMap::new(),
		}
	}
}

impl<R> Default for Layouts<R> {
	fn default() -> Self {
		Self::new()
	}
}

impl<R: Ord> Layouts<R> {
	pub fn layout(&self, id: &R) -> Option<&Layout<R>> {
		self.layouts.get(id)
	}

	pub fn get<T: GetFromLayouts<Self, R>>(&self, r: &Ref<T, R>) -> Option<T::Target<'_>> {
		T::get_from_layouts(self, r)
	}
}

impl<R: Clone + Ord> Layouts<R> {
	pub fn insert(&mut self, id: R, layout: Layout<R>) -> (Ref<LayoutType, R>, Option<Layout<R>>) {
		self.insert_with(id, |_| layout)
	}

	pub fn insert_with(
		&mut self,
		id: R,
		builder: impl FnOnce(&Ref<LayoutType, R>) -> Layout<R>,
	) -> (Ref<LayoutType, R>, Option<Layout<R>>) {
		let layout_ref = Ref::new(id.clone());
		let layout = builder(&layout_ref);

		let old_layout = self.layouts.insert(id, layout);

		(layout_ref, old_layout)
	}
}
