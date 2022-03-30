use crate::{prop, Causes, Documentation, Id};
use shelves::Ref;

pub mod normal;

pub use normal::Normal;

/// Type definition.
pub struct Definition<F> {
	/// Identifier.
	id: Id,

	/// Causes of the definition.
	causes: Causes<F>,

	/// Documentation.
	doc: Documentation,

	/// Type description.
	desc: Description<F>
}

/// Type definition.
pub enum Description<F> {
	Normal(Normal<F>)
}

impl<F> Definition<F> {
	pub fn new(id: Id, desc: Description<F>, causes: impl Into<Causes<F>>) -> Self {
		Self {
			id,
			causes: causes.into(),
			doc: Documentation::default(),
			desc
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

	pub fn properties(&self) -> Properties<F> {
		match &self.desc {
			Description::Normal(n) => Properties::Normal(n.properties())
		}
	}
}

/// Iterator over the properties of a type.
pub enum Properties<'a, F> {
	Normal(normal::Properties<'a, F>)
}

impl<'a, F> Iterator for Properties<'a, F> {
	type Item = (Ref<prop::Definition<F>>, &'a Causes<F>);

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Normal(n) => n.next()
		}
	}
}

// #[derive(Derivative)]
// #[derivative(Clone(bound = ""), Copy(bound = ""))]
// pub struct Expr<F> {
// 	ty_ref: Ref<Definition<F>>,
// 	implicit_layout_ref: Option<Ref<layout::Definition<F>>>,
// }

// impl<F> Expr<F> {
// 	pub fn new(
// 		ty_ref: Ref<Definition<F>>,
// 		implicit_layout_ref: Option<Ref<layout::Definition<F>>>,
// 	) -> Self {
// 		Self {
// 			ty_ref,
// 			implicit_layout_ref,
// 		}
// 	}

// 	pub fn ty(&self) -> Ref<Definition<F>> {
// 		self.ty_ref
// 	}

// 	pub fn implicit_layout(&self) -> Option<Ref<layout::Definition<F>>> {
// 		self.implicit_layout_ref
// 	}

// 	pub fn set_implicit_layout(&mut self, l: Ref<layout::Definition<F>>) {
// 		self.implicit_layout_ref = Some(l)
// 	}
// }
