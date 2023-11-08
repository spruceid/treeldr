pub mod layout;
pub mod regexp;
pub mod syntax;

use std::collections::BTreeMap;

pub use layout::Layout;
use rdf_types::Interpretation;
pub use regexp::RegExp;
use treeldr::{layout::LayoutType, Ref};

pub struct Builder<R> {
	layouts: BTreeMap<R, Layout<R>>,
}

impl<R> Builder<R> {
	pub fn new() -> Self {
		Self {
			layouts: BTreeMap::new(),
		}
	}

	pub fn with_interpretation_mut<'a, V, I: Interpretation<Resource = R>>(
		&'a mut self,
		vocabulary: &'a mut V,
		interpretation: &'a mut I,
	) -> BuilderWithInterpretationMut<'a, V, I> {
		BuilderWithInterpretationMut {
			vocabulary,
			interpretation,
			builder: self,
		}
	}
}

pub type InsertResult<R> = (Ref<LayoutType, R>, Option<Layout<R>>);

impl<R: Clone + Eq + Ord> Builder<R> {
	pub fn insert(&mut self, id: R, layout: Layout<R>) -> InsertResult<R> {
		self.insert_with(id, |_| layout)
	}

	pub fn insert_with(
		&mut self,
		id: R,
		builder: impl FnOnce(&Ref<LayoutType, R>) -> Layout<R>,
	) -> InsertResult<R> {
		let layout_ref = Ref::new(id.clone());
		let layout = builder(&layout_ref);

		let old_layout = self.layouts.insert(id, layout);

		(layout_ref, old_layout)
	}

	pub fn get_or_insert_with(
		&mut self,
		layout_ref: Ref<LayoutType, R>,
		builder: impl FnOnce(&Ref<LayoutType, R>) -> Layout<R>,
	) -> &Layout<R> {
		self.layouts
			.entry(layout_ref.into_id())
			.or_insert_with_key(|id| builder(Ref::new_ref(id)))
	}

	pub fn build(&self) -> treeldr::Layouts<R> {
		let mut result = treeldr::Layouts::new();

		for (id, layout) in &self.layouts {
			result.insert(id.clone(), layout.build());
		}

		result
	}
}

impl<R> Default for Builder<R> {
	fn default() -> Self {
		Self::new()
	}
}

pub struct BuilderWithInterpretationMut<'a, V, I: Interpretation> {
	vocabulary: &'a mut V,
	interpretation: &'a mut I,
	builder: &'a mut Builder<I::Resource>,
}
