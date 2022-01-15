use crate::{
	Id,
	source::Causes
};
use std::fmt;

/// Type definition.
pub struct Definition {
	id: Id,
	causes: Causes
}

impl Definition {
	pub fn new(id: Id, causes: impl Into<Causes>) -> Self {
		Self {
			id,
			causes: causes.into()
		}
	}

	/// Returns the identifier of the defined type.
	pub fn id(&self) -> Id {
		self.id
	}

	pub fn causes(&self) -> &Causes {
		&self.causes
	}
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Expr {
	ty: crate::Ref<Definition>,
	args: Vec<Self>
}

impl Expr {
	pub fn new(ty: crate::Ref<Definition>, args: Vec<Self>) -> Self {
		Self {
			ty,
			args
		}
	}

	pub fn with_context<'c>(&self, context: &'c crate::Context) -> ExprWithContext<'c, '_> {
		ExprWithContext(context, self)
	}
}

pub struct ExprWithContext<'c, 'e>(&'c crate::Context, &'e Expr);

impl<'c, 'e> ExprWithContext<'c, 'e> {
	pub fn context(&self) -> &'c crate::Context {
		self.0
	}

	pub fn expr(&self) -> &'e Expr {
		self.1
	}
}

impl<'c, 'e> fmt::Display for ExprWithContext<'c, 'e> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let ty_def = self.context().types().get(self.expr().ty).unwrap();
		let iri = self.context().vocabulary().get(ty_def.id).unwrap();

		iri.fmt(f)?;

		if !self.expr().args.is_empty() {
			write!(f, "(")?;
			for (i, arg) in self.expr().args.iter().enumerate() {
				if i > 0 {
					write!(f, ", ")?;
				}

				arg.with_context(self.context()).fmt(f)?;
			}
			write!(f, ")")?;
		}

		Ok(())
	}
}