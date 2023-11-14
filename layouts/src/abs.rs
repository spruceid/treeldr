//! Abstract syntax implementation for layouts.
pub mod layout;
pub mod regexp;
pub mod syntax;

use std::collections::BTreeMap;

use crate::{layout::LayoutType, Ref};
pub use layout::Layout;
use rdf_types::Interpretation;
pub use regexp::RegExp;

/// Layout builder.
///
/// Stores all the pre-built layouts. Can be used to build a
/// [`Layouts`](crate::Layouts) collection using the [`build`](Self::build)
/// method.
pub struct Builder<R = rdf_types::Term> {
	/// Pre-built layouts.
	layouts: BTreeMap<R, Layout<R>>,
}

impl<R> Builder<R> {
	/// Creates a new empty layout builder.
	pub fn new() -> Self {
		Self {
			layouts: BTreeMap::new(),
		}
	}

	/// Borrows the builder with an RDF interpretation.
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

impl Builder {
	/// Borrows the builder with a the lexical RDF interpretation (`()`)
	/// combined with a node identifier generator.
	pub fn with_generator_mut<G>(&mut self, generator: G) -> BuilderWithGeneratorMut<G> {
		BuilderWithGeneratorMut {
			builder: self,
			generator,
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

	pub fn build(&self) -> crate::Layouts<R> {
		let mut result = crate::Layouts::new();

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

pub struct BuilderWithGeneratorMut<'a, G> {
	builder: &'a mut Builder,
	generator: G,
}

pub struct BuilderWithInterpretationMut<'a, V, I: Interpretation> {
	vocabulary: &'a mut V,
	interpretation: &'a mut I,
	builder: &'a mut Builder<I::Resource>,
}
