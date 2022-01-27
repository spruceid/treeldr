use crate::{
	Error,
	Ref,
	Id,
	Cause,
	Caused,
	Causes,
	Documentation,
	prop,
	layout
};
use iref::Iri;
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

	pub fn properties(&self) -> impl Iterator<Item=(Ref<prop::Definition>, &Causes)> {
		self.properties.iter().map(|(p, c)| (*p, c))
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

	pub fn default_fields(&self, model: &crate::Model) -> Result<Vec<layout::Field>, Caused<Error>> {
		struct PropertyIri<'a>(Iri<'a>, Ref<prop::Definition>, &'a Causes);

		impl<'a> PartialEq for PropertyIri<'a> {
			fn eq(&self, other: &Self) -> bool {
				self.0 == other.0
			}
		}

		impl<'a> Eq for PropertyIri<'a> {}

		impl<'a> std::cmp::Ord for PropertyIri<'a> {
			fn cmp(&self, other: &Self) -> std::cmp::Ordering {
				self.0.cmp(&other.0)
			}
		}

		impl<'a> std::cmp::PartialOrd for PropertyIri<'a> {
			fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
				self.0.partial_cmp(&other.0)
			}
		}

		let mut properties: Vec<PropertyIri> = self.properties.iter().map(|(prop_ref, causes)| {
			let prop = model.properties().get(*prop_ref).unwrap();
			let iri = model.vocabulary().get(prop.id()).unwrap();
			PropertyIri(iri, *prop_ref, causes)
		}).collect();

		properties.sort();

		let mut fields = Vec::with_capacity(properties.len());
		for PropertyIri(iri, prop_ref, causes) in properties {
			let prop = model.properties().get(prop_ref).unwrap();
			let name = iri.path().file_name().expect("invalid property IRI").to_owned();
			let layout_expr = match prop.ty() {
				Some(ty) => ty.expr().as_layout_expr(model, ty.causes().preferred().map(Cause::into_implicit))?,
				None => panic!("no known type")
			};

			let mut field = layout::Field::new(prop_ref, name, layout_expr, causes.map(Cause::into_implicit));

			field.set_required(prop.is_required());
			field.set_functional(prop.is_functional());

			fields.push(field);
		}

		Ok(fields)
	}
}

impl Ref<Definition> {
	pub fn with_model<'c>(&self, context: &'c crate::Model) -> RefWithContext<'c> {
		RefWithContext(context, *self)
	}
}

pub struct RefWithContext<'c>(&'c crate::Model, Ref<Definition>);

impl<'c> fmt::Display for RefWithContext<'c> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let id = self.0.types().get(self.1).unwrap().id();
		let iri = self.0.vocabulary().get(id).unwrap();
		iri.fmt(f)
	}
}

/// Type expression.
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

	pub fn as_layout_expr(&self, model: &crate::Model, cause: Option<Cause>) -> Result<layout::Expr, Caused<Error>> {
		let id = model.types().get(self.ty).unwrap().id();
		let layout_ref = model.require_layout(id, cause)?;
		let mut args = Vec::with_capacity(self.args.len());
		for arg in &self.args {
			args.push(arg.as_layout_expr(model, cause)?)
		}
		Ok(layout::Expr::new(layout_ref, args))
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