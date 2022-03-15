use crate::{ty, Caused, Causes, Documentation, Error, Id, Ref};
use std::collections::HashMap;
use locspan::Location;

/// Property definition.
pub struct Definition<F> {
	id: Id,
	causes: Causes<F>,
	domain: HashMap<Ref<ty::Definition<F>>, Causes<F>>,
	ty: Option<Type<F>>,
	required: bool,
	functional: bool,
	doc: Documentation,
}

impl<F> Definition<F> {
	pub fn new(id: Id, causes: impl Into<Causes<F>>) -> Self {
		Self {
			id,
			causes: causes.into(),
			domain: HashMap::new(),
			ty: None,
			required: false,
			functional: true,
			doc: Documentation::default(),
		}
	}

	/// Returns the identifier of the defined property.
	pub fn id(&self) -> Id {
		self.id
	}

	pub fn causes(&self) -> &Causes<F> {
		&self.causes
	}

	pub fn ty(&self) -> Option<&Type<F>> {
		self.ty.as_ref()
	}

	pub fn is_required(&self) -> bool {
		self.required
	}

	pub fn declare_required(&mut self) {
		self.required = true
	}

	/// Checks if this property is functional,
	/// meaning that it is associated to at most one value.
	pub fn is_functional(&self) -> bool {
		self.functional
	}

	pub fn declare_functional(&mut self) {
		self.functional = true
	}

	pub fn declare_multiple(&mut self) {
		self.functional = false
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

	pub fn declare_domain(&mut self, ty_ref: Ref<ty::Definition<F>>, cause: Option<Location<F>>) where F: Ord {
		use std::collections::hash_map::Entry;
		match self.domain.entry(ty_ref) {
			Entry::Vacant(entry) => {
				entry.insert(cause.into());
			}
			Entry::Occupied(mut entry) => {
				if let Some(cause) = cause {
					entry.get_mut().add(cause)
				}
			}
		}
	}

	pub fn declare_type(
		&mut self,
		ty_expr: ty::Expr<F>,
		cause: Option<Location<F>>,
	) -> Result<(), Caused<Error<F>, F>> where F: Clone + Ord {
		match &mut self.ty {
			Some(ty) => ty.declare(ty_expr, cause),
			None => {
				self.ty = Some(Type::new(ty_expr, cause));
				Ok(())
			}
		}
	}
}

pub struct Type<F> {
	ty_expr: ty::Expr<F>,
	causes: Causes<F>,
}

impl<F> Type<F> {
	pub fn new(ty_expr: ty::Expr<F>, causes: impl Into<Causes<F>>) -> Self {
		Self {
			ty_expr,
			causes: causes.into(),
		}
	}

	pub fn expr(&self) -> ty::Expr<F> {
		self.ty_expr
	}

	pub fn causes(&self) -> &Causes<F> {
		&self.causes
	}

	pub fn declare(
		&mut self,
		ty_expr: ty::Expr<F>,
		source: Option<Location<F>>,
	) -> Result<(), Caused<Error<F>, F>> where F: Clone {
		if self.ty_expr.ty() == ty_expr.ty() {
			match (self.ty_expr.implicit_layout(), ty_expr.implicit_layout()) {
				(Some(expected), Some(found)) => {
					if expected != found {
						return Err(Caused::new(
							Error::ImplicitLayoutMismatch {
								expected,
								found,
								because: self.causes.preferred().cloned(),
							},
							source,
						));
					}
				}
				(None, Some(b)) => self.ty_expr.set_implicit_layout(b),
				_ => (),
			}

			Ok(())
		} else {
			Err(Caused::new(
				Error::TypeMismatch {
					expected: self.ty_expr.ty(),
					found: ty_expr.ty(),
					because: self.causes.preferred().cloned(),
				},
				source,
			))
		}
	}
}
