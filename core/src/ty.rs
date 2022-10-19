use crate::{BlankIdIndex, Documentation, Id, IriIndex, Model};

pub mod data;
mod intersection;
pub mod normal;
pub mod properties;
pub mod restriction;
mod r#union;

pub use data::DataType;
pub use intersection::Intersection;
pub use normal::Normal;
pub use properties::{Properties, PseudoProperty};
use rdf_types::Subject;
pub use restriction::Restriction;
pub use union::Union;

/// Type definition.
pub struct Definition<M, I = IriIndex, B = BlankIdIndex> {
	/// Identifier.
	id: Subject<I, B>,

	/// Metadata of the definition.
	metadata: M,

	/// Type description.
	desc: Description<M>,
}

/// Type definition.
pub enum Description<M> {
	Empty,
	Data(DataType),
	Normal(Normal<M>),
	Union(Union<M>),
	Intersection(Intersection<M>),
	Restriction(Restriction<M>),
}

impl<M> Description<M> {
	pub fn kind(&self) -> Kind {
		match self {
			Self::Empty => Kind::Empty,
			Self::Data(_) => Kind::Data,
			Self::Normal(_) => Kind::Normal,
			Self::Union(_) => Kind::Union,
			Self::Intersection(_) => Kind::Intersection,
			Self::Restriction(_) => Kind::Restriction,
		}
	}

	pub fn is_datatype(&self, model: &Model<M>) -> bool {
		match self {
			Self::Data(_) => true,
			Self::Union(u) => u.is_datatype(model),
			Self::Intersection(i) => i.is_datatype(model),
			_ => false,
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Kind {
	Empty,
	Data,
	Normal,
	Union,
	Intersection,
	Restriction,
}

impl<M> Definition<M> {
	pub fn new(id: Id, desc: Description<M>, causes: impl Into<M>) -> Self {
		Self {
			id,
			metadata: causes.into(),
			desc,
		}
	}

	/// Returns the identifier of the defined type.
	pub fn id(&self) -> Id {
		self.id
	}

	pub fn causes(&self) -> &M {
		&self.metadata
	}

	pub fn description(&self) -> &Description<M> {
		&self.desc
	}

	pub fn label<'m>(&self, model: &'m crate::Model<M>) -> Option<&'m str> {
		model.get(self.id).unwrap().label()
	}

	pub fn documentation<'m>(&self, model: &'m crate::Model<M>) -> &'m Documentation {
		model.get(self.id).unwrap().documentation()
	}

	pub fn properties(&self) -> Option<&Properties<M>> {
		match &self.desc {
			Description::Empty => None,
			Description::Data(_) => None,
			Description::Normal(n) => Some(n.properties()),
			Description::Union(u) => Some(u.properties()),
			Description::Intersection(i) => Some(i.properties()),
			Description::Restriction(r) => Some(r.properties()),
		}
	}

	pub fn is_datatype(&self, model: &Model<M>) -> bool {
		self.desc.is_datatype(model)
	}
}
