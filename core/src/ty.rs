use crate::{prop, Causes, Documentation, Id, Model};
use shelves::Ref;

mod intersection;
pub mod normal;
mod r#union;
pub mod restriction;

pub use intersection::Intersection;
pub use normal::Normal;
pub use union::Union;
pub use restriction::Restricted;

/// Type definition.
pub struct Definition<F> {
	/// Identifier.
	id: Id,

	/// Causes of the definition.
	causes: Causes<F>,

	/// Documentation.
	doc: Documentation,

	/// Type description.
	desc: Description<F>,
}

/// Type definition.
pub enum Description<F> {
	Normal(Normal<F>),
	Union(Union<F>),
	Intersection(Intersection<F>),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Kind {
	Normal,
	Union,
	Intersection,
}

impl<F> Definition<F> {
	pub fn new(id: Id, desc: Description<F>, causes: impl Into<Causes<F>>) -> Self {
		Self {
			id,
			causes: causes.into(),
			doc: Documentation::default(),
			desc,
		}
	}

	/// Returns the identifier of the defined type.
	pub fn id(&self) -> Id {
		self.id
	}

	pub fn causes(&self) -> &Causes<F> {
		&self.causes
	}

	pub fn documentation(&self) -> &Documentation {
		&self.doc
	}

	pub fn documentation_mut(&mut self) -> &mut Documentation {
		&mut self.doc
	}

	pub fn set_documentation(&mut self, doc: Documentation) {
		self.doc = doc
	}

	pub fn properties<'m>(&'m self, model: &'m Model<F>) -> Properties<'m, F>
	where
		F: Clone + Ord,
	{
		match &self.desc {
			Description::Normal(n) => n.properties(),
			Description::Union(u) => u.properties(model),
			Description::Intersection(i) => i.properties(model),
		}
	}
}

pub struct Properties<'a, F>(
	std::collections::hash_map::Iter<'a, Ref<prop::Definition<F>>, Causes<F>>,
);

impl<'a, F> Iterator for Properties<'a, F> {
	type Item = (Ref<prop::Definition<F>>, &'a Causes<F>);

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next().map(|(r, c)| (*r, c))
	}
}
