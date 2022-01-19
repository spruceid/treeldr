use crate::{
	Ref,
	Id,
	Cause,
	source::Causes,
	Documentation,
	prop
};
use std::fmt;
use std::collections::HashMap;

/// Type definition.
pub struct Definition {
	/// Identifier.
	id: Id,

	/// Causes of the definition.
	causes: Causes,

	/// Properties.
	properties: HashMap<Ref<prop::Definition>, Causes>,

	/// Documentation.
	doc: Documentation
}

impl Definition {
	pub fn new(id: Id, causes: impl Into<Causes>) -> Self {
		Self {
			id,
			causes: causes.into(),
			properties: HashMap::new(),
			doc: Documentation::default()
		}
	}

	/// Returns the identifier of the defined type.
	pub fn id(&self) -> Id {
		self.id
	}

	pub fn causes(&self) -> &Causes {
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

	pub fn declare_property(&mut self, prop_ref: Ref<prop::Definition>, cause: Option<Cause>) {
		use std::collections::hash_map::Entry;
		match self.properties.entry(prop_ref) {
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

	pub fn ty(&self) -> crate::Ref<Definition> {
		self.ty
	}
	
	pub fn arguments(&self) -> &[Self] {
		&self.args
	}

	pub fn with_model<'c>(&self, context: &'c crate::Model) -> ExprWithContext<'c, '_> {
		ExprWithContext(context, self)
	}
}

pub struct ExprWithContext<'c, 'e>(&'c crate::Model, &'e Expr);

impl<'c, 'e> ExprWithContext<'c, 'e> {
	pub fn context(&self) -> &'c crate::Model {
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

				arg.with_model(self.context()).fmt(f)?;
			}
			write!(f, ")")?;
		}

		Ok(())
	}
}