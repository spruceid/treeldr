pub mod de;
pub mod hy;

pub use hy::{hydrate, hydrate_with};

pub struct RdfContext<'a, V, I> {
	pub vocabulary: &'a V,
	pub interpretation: &'a I,
}

impl<'a, V, I> RdfContext<'a, V, I> {
	pub fn new(vocabulary: &'a V, interpretation: &'a I) -> Self {
		Self {
			vocabulary,
			interpretation,
		}
	}
}

pub struct RdfContextMut<'a, V, I> {
	pub vocabulary: &'a mut V,
	pub interpretation: &'a mut I,
}

impl<'a, V, I> RdfContextMut<'a, V, I> {
	pub fn new(vocabulary: &'a mut V, interpretation: &'a mut I) -> Self {
		Self {
			vocabulary,
			interpretation,
		}
	}
}
