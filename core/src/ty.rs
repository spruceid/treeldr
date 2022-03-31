use crate::{prop, Causes, Documentation, Id, Model};
use shelves::Ref;

pub mod normal;
mod r#union;

pub use normal::Normal;
pub use union::Union;

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
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Kind {
	Normal,
	Union,
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

	pub fn properties_with_duplicates<'m>(
		&'m self,
		model: &'m Model<F>,
	) -> PropertiesWithDuplicates<'m, F> {
		match &self.desc {
			Description::Normal(n) => PropertiesWithDuplicates::Normal(n.properties()),
			Description::Union(u) => {
				PropertiesWithDuplicates::Union(u.properties_with_duplicates(model))
			}
		}
	}

	pub fn properties<'m>(&'m self, model: &'m Model<F>) -> Properties<'m, F> {
		match &self.desc {
			Description::Normal(n) => Properties::Normal(n.properties()),
			Description::Union(u) => Properties::Union(u.properties(model)),
		}
	}
}

/// Iterator over the properties of a type.
pub enum PropertiesWithDuplicates<'a, F> {
	Normal(normal::Properties<'a, F>),
	Union(union::PropertiesWithDuplicates<'a, F>),
}

impl<'a, F> Iterator for PropertiesWithDuplicates<'a, F> {
	type Item = (Ref<prop::Definition<F>>, &'a Causes<F>);

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Normal(n) => n.next(),
			Self::Union(u) => u.next(),
		}
	}
}

/// Iterator over the properties of a type.
pub enum Properties<'a, F> {
	Normal(normal::Properties<'a, F>),
	Union(union::Properties<'a, F>),
}

impl<'a, F> Iterator for Properties<'a, F> {
	type Item = (Ref<prop::Definition<F>>, &'a Causes<F>);

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Normal(n) => n.next(),
			Self::Union(u) => u.next(),
		}
	}
}
