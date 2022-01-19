use crate::{
	Error,
	Ref,
	Id,
	Cause,
	Caused,
	source::Causes,
	Documentation,
	ty
};
use std::collections::HashMap;

/// Property definition.
pub struct Definition {
	id: Id,
	causes: Causes,
	domain: HashMap<Ref<ty::Definition>, Causes>,
	ty: Option<Type>,
	doc: Documentation
}

impl Definition {
	pub fn new(id: Id, causes: impl Into<Causes>) -> Self {
		Self {
			id,
			causes: causes.into(),
			domain: HashMap::new(),
			ty: None,
			doc: Documentation::default()
		}
	}

	/// Returns the identifier of the defined property.
	pub fn id(&self) -> Id {
		self.id
	}

	pub fn causes(&self) -> &Causes {
		&self.causes
	}

	pub fn ty(&self) -> Option<&Type> {
		self.ty.as_ref()
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

	pub fn declare_domain(&mut self, ty_ref: Ref<ty::Definition>, cause: Option<Cause>) {
		use std::collections::hash_map::Entry;
		match self.domain.entry(ty_ref) {
			Entry::Vacant(entry) => {
				entry.insert(cause.into());
			},
			Entry::Occupied(mut entry) => {
				if let Some(cause) = cause {
					entry.get_mut().add(cause)
				}
			}
		}
	}

	pub fn declare_type(&mut self, ty_expr: ty::Expr, cause: Option<Cause>) -> Result<(), Caused<Error>> {
		match &mut self.ty {
			Some(ty) => ty.declare(ty_expr, cause),
			None => {
				self.ty = Some(Type::new(ty_expr, cause));
				Ok(())
			}
		}
	}
}

pub struct Type {
	expr: ty::Expr,
	causes: Causes
}

impl Type {
	pub fn new(expr: ty::Expr, causes: impl Into<Causes>) -> Self {
		Self {
			expr,
			causes: causes.into()
		}
	}

	pub fn expr(&self) -> &ty::Expr {
		&self.expr
	}

	pub fn causes(&self) -> &Causes {
		&self.causes
	}

	pub fn declare(&mut self, ty_expr: ty::Expr, source: Option<Cause>) -> Result<(), Caused<Error>> {
		if self.expr == ty_expr {
			Ok(())
		} else {
			Err(Caused::new(Error::TypeMismatch { expected: self.expr.clone(), found: ty_expr, because: self.causes.preferred() }, source))
		}
	}
}