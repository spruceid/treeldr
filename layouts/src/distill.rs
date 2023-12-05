pub mod de;
pub mod hy;

pub use de::{dehydrate, dehydrate_with};
pub use hy::{hydrate, hydrate_with};

/// RDF context, providing the RDF vocabulary and interpretation.
pub struct RdfContext<'a, V, I> {
	/// Vocabulary storing the lexical representations of terms.
	pub vocabulary: &'a V,

	/// RDF interpretation, mapping resources to terms.
	pub interpretation: &'a I,
}

impl<'a, V, I> RdfContext<'a, V, I> {
	/// Creates a new RDF context.
	pub fn new(vocabulary: &'a V, interpretation: &'a I) -> Self {
		Self {
			vocabulary,
			interpretation,
		}
	}
}

/// Mutable RDF context, providing the mutable RDF vocabulary and
/// interpretation.
pub struct RdfContextMut<'a, V, I> {
	/// Vocabulary storing the lexical representations of terms.
	pub vocabulary: &'a mut V,

	/// RDF interpretation, mapping resources to terms.
	pub interpretation: &'a mut I,
}

impl<'a, V, I> RdfContextMut<'a, V, I> {
	/// Creates a new mutable RDF context.
	pub fn new(vocabulary: &'a mut V, interpretation: &'a mut I) -> Self {
		Self {
			vocabulary,
			interpretation,
		}
	}
}
