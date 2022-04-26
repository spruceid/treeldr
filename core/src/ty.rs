use crate::{Causes, Documentation, Id};

pub mod properties;
mod intersection;
pub mod normal;
pub mod restriction;
mod r#union;

pub use properties::{Properties, PseudoProperty};
pub use intersection::Intersection;
pub use normal::Normal;
pub use restriction::Restriction;
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
	Empty,
	Normal(Normal<F>),
	Union(Union<F>),
	Intersection(Intersection<F>),
	Restriction(Restriction<F>)
}

impl<F> Description<F> {
	pub fn kind(&self) -> Kind {
		match self {
			Self::Empty => Kind::Empty,
			Self::Normal(_) => Kind::Normal,
			Self::Union(_) => Kind::Union,
			Self::Intersection(_) => Kind::Intersection,
			Self::Restriction(_) => Kind::Restriction
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Kind {
	Empty,
	Normal,
	Union,
	Intersection,
	Restriction
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

	pub fn properties(&self) -> Option<&Properties<F>> {
		match &self.desc {
			Description::Empty => None,
			Description::Normal(n) => Some(n.properties()),
			Description::Union(u) => Some(u.properties()),
			Description::Intersection(i) => Some(i.properties()),
			Description::Restriction(r) => Some(r.properties())
		}
	}
}