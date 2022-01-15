use crate::{
	Error,
	Ref,
	Id,
	Cause,
	Caused,
	source::Causes,
	ty
};
use std::fmt;

/// Layout definition.
pub struct Definition {
	id: Id,
	ty: Option<Type>,
	causes: Causes
}

impl Definition {
	pub fn new(id: Id, causes: impl Into<Causes>) -> Self {
		Self {
			id,
			ty: None,
			causes: causes.into()
		}
	}

	/// Type for which the layout is defined.
	pub fn ty(&self) -> Option<&Type> {
		self.ty.as_ref()
	}

	/// Returns the identifier of the defined layout.
	pub fn id(&self) -> Id {
		self.id
	}

	pub fn causes(&self) -> &Causes {
		&self.causes
	}

	/// Declare the type for which this layout is defined.
	pub fn declare_type(&mut self, ty_ref: Ref<ty::Definition>, cause: Option<Cause>) -> Result<(), Caused<Error>> {
		match &self.ty {
			Some(expected_ty) => {
				if expected_ty.reference() != ty_ref {
					return Err(Caused::new(Error::LayoutTypeMismatch { expected: expected_ty.reference(), found: ty_ref, because: expected_ty.causes().preferred() }, cause))
				}
			},
			None => {
				self.ty = Some(Type::new(ty_ref, cause));
			}
		}

		Ok(())
	}
}

pub struct Type {
	ty_ref: Ref<ty::Definition>,
	causes: Causes
}

impl Type {
	pub fn new(ty_ref: Ref<ty::Definition>, causes: impl Into<Causes>) -> Self {
		Self {
			ty_ref,
			causes: causes.into()
		}
	}

	pub fn reference(&self) -> Ref<ty::Definition> {
		self.ty_ref
	}

	pub fn causes(&self) -> &Causes {
		&self.causes
	}

	pub fn with_context<'c>(&self, context: &'c crate::Context) -> TypeWithContext<'c, '_> {
		TypeWithContext(context, self)
	}
}

pub struct TypeWithContext<'c, 't>(&'c crate::Context, &'t Type);

impl<'c, 't> fmt::Display for TypeWithContext<'c, 't> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let id = self.0.types().get(self.1.reference()).unwrap().id();
		let iri = self.0.vocabulary().get(id).unwrap();
		iri.fmt(f)
	}
}