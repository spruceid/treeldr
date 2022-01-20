use crate::{
	Error,
	Ref,
	Id,
	Cause,
	Caused,
	WithCauses,
	source::Causes,
	Documentation,
	ty,
	prop,
	layout
};
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Type {
	Struct,
	Native(Native)
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Native {
	Boolean,
	Integer,
	PositiveInteger,
	Float,
	Double,
	String,
	Time,
	Date,
	DateTime,
	Iri,
	Uri,
	Url
}

/// Layout definition.
pub struct Definition {
	id: Id,
	ty: Option<WithCauses<Ref<ty::Definition>>>,
	causes: Causes,
	doc: Documentation,
	desc: Option<WithCauses<Description>>,
}

pub enum Description {
	Struct(Fields),
	Native(Native)
}

impl Description {
	pub fn ty(&self) -> Type {
		match self {
			Self::Struct(_) => Type::Struct,
			Self::Native(n) => Type::Native(*n)
		}
	}
}

impl Definition {
	pub fn new(id: Id, causes: impl Into<Causes>) -> Self {
		Self {
			id,
			ty: None,
			causes: causes.into(),
			doc: Documentation::default(),
			desc: None
		}
	}

	/// Type for which the layout is defined.
	pub fn ty(&self) -> Option<&WithCauses<Ref<ty::Definition>>> {
		self.ty.as_ref()
	}

	/// Returns the identifier of the defined layout.
	pub fn id(&self) -> Id {
		self.id
	}

	pub fn causes(&self) -> &Causes {
		&self.causes
	}

	pub fn description(&self) -> Option<&WithCauses<Description>> {
		self.desc.as_ref()
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

	pub fn preferred_documentation<'a>(&'a self, model: &'a crate::Model) -> &'a Documentation {
		if self.doc.is_empty() {
			match &self.ty {
				Some(ty) => model.types().get(*ty.inner()).unwrap().documentation(),
				None => &self.doc
			}
		} else {
			&self.doc
		}
	}

	/// Declare the type for which this layout is defined.
	pub fn declare_type(&mut self, ty_ref: Ref<ty::Definition>, cause: Option<Cause>) -> Result<(), Caused<Error>> {
		match &self.ty {
			Some(expected_ty) => {
				if *expected_ty.inner() != ty_ref {
					return Err(Caused::new(Error::LayoutTypeMismatch { expected: *expected_ty.inner(), found: ty_ref, because: expected_ty.causes().preferred() }, cause))
				}
			},
			None => {
				self.ty = Some(WithCauses::new(ty_ref, cause));
			}
		}

		Ok(())
	}

	pub fn declare_native(&mut self, native: Native, cause: Option<Cause>) -> Result<(), Caused<Mismatch>> {
		match &mut self.desc {
			Some(desc) => {
				match desc.inner_mut() {
					Description::Native(n) if *n == native => Ok(()),
					_ => Err(Caused::new(Mismatch::Type { expected: desc.ty(), found: Type::Struct, because: desc.causes().preferred() }, cause))
				}
			}
			None => {
				self.desc = Some(WithCauses::new(Description::Native(native), cause));
				Ok(())
			}
		}
	}

	pub fn declare_fields(&mut self, fields: Vec<Field>, cause: Option<Cause>) -> Result<(), Caused<Mismatch>> {
		match &mut self.desc {
			Some(desc) => {
				let desc_cause = desc.causes().preferred();
				match desc.inner_mut() {
					Description::Struct(current_fields) => current_fields.add_causes(desc_cause, &fields, cause),
					_ => Err(Caused::new(Mismatch::Type { expected: desc.ty(), found: Type::Struct, because: desc.causes().preferred() }, cause))
				}
			}
			None => {
				self.desc = Some(WithCauses::new(Description::Struct(Fields::new(fields)), cause));
				Ok(())
			}
		}
	}

	pub fn set_fields(&mut self, fields: Vec<Field>, causes: impl Into<Causes>) {
		self.desc = Some(WithCauses::new(Description::Struct(Fields::new(fields)), causes))
	}
}

/// Layout mismatch error.
#[derive(Debug)]
pub enum Mismatch {
	Type {
		expected: Type,
		found: Type,
		because: Option<Cause>
	},
	FieldProperty {
		expected: Ref<prop::Definition>,
		found: Ref<prop::Definition>,
		because: Option<Cause>
	},
	FieldName {
		expected: String,
		found: String,
		because: Option<Cause>
	},
	FieldLayout {
		expected: layout::Expr,
		found: layout::Expr,
		because: Option<Cause>
	},
	MissingField {
		name: String,
		because: Option<Cause>
	},
	AdditionalField {
		name: String,
		because: Option<Cause>
	}
}

/// Layout fields.
pub struct Fields {
	fields: Vec<Field>
}

impl Fields {
	pub fn new(fields: Vec<Field>) -> Self {
		Self {
			fields
		}
	}

	pub fn as_slice(&self) -> &[Field] {
		&self.fields
	}

	pub fn iter(&self) -> std::slice::Iter<Field> {
		self.fields.iter()
	}
	
	pub fn add_causes(&mut self, self_cause: Option<Cause>, fields: &[Field], cause: Option<Cause>) -> Result<(), Caused<Mismatch>> {
		for (a, b) in self.fields.iter().zip(fields) {
			if a.property() != b.property() {
				return Err(Caused::new(
					Mismatch::FieldProperty {
						expected: a.property(),
						found: b.property(),
						because: a.causes().preferred()
					},
					b.causes().preferred()
				))
			}

			if a.name() != b.name() {
				return Err(Caused::new(
					Mismatch::FieldName {
						expected: a.name().to_owned(),
						found: b.name().to_owned(),
						because: a.causes().preferred()
					},
					b.causes().preferred()
				))
			}

			if a.layout() != b.layout() {
				return Err(Caused::new(
					Mismatch::FieldLayout {
						expected: a.layout().clone(),
						found: b.layout().clone(),
						because: a.causes().preferred()
					},
					b.causes().preferred()
				))
			}
		}

		if self.fields.len() > fields.len() {
			let field = &self.fields[fields.len()];
			return Err(Caused::new(
				Mismatch::MissingField {
					name: field.name().to_owned(),
					because: field.causes().preferred()
				}, cause
			))
		}

		if fields.len() > self.fields.len() {
			let field = &fields[self.fields.len()];
			return Err(Caused::new(
				Mismatch::AdditionalField {
					name: field.name().to_owned(),
					because: self_cause
				},
				field.causes().preferred()
			))
		}

		Ok(())
	}
}

impl AsRef<[Field]> for Fields {
	fn as_ref(&self) -> &[Field] {
		self.as_slice()
	}
}

impl std::ops::Deref for Fields {
	type Target = [Field];

	fn deref(&self) -> &[Field] {
		self.as_slice()
	}
}

impl<'a> IntoIterator for &'a Fields {
	type Item = &'a Field;
	type IntoIter = std::slice::Iter<'a, Field>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

/// Layout field.
pub struct Field {
	prop: Ref<prop::Definition>,
	name: String,
	layout: Expr,
	causes: Causes,
	doc: Documentation
}

impl Field {
	pub fn new(prop: Ref<prop::Definition>, name: String, layout: Expr, causes: impl Into<Causes>) -> Self {
		Self {
			prop,
			name,
			layout,
			causes: causes.into(),
			doc: Documentation::default()
		}
	}
	
	pub fn property(&self) -> Ref<prop::Definition> {
		self.prop
	}

	pub fn name(&self) -> &str {
		&self.name
	}

	pub fn layout(&self) -> &Expr {
		&self.layout
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

	pub fn preferred_documentation<'a>(&'a self, model: &'a crate::Model) -> &'a Documentation {
		if self.doc.is_empty() {
			model.properties().get(self.prop).unwrap().documentation()
		} else {
			&self.doc
		}
	}
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Expr {
	layout: crate::Ref<Definition>,
	args: Vec<Self>
}

impl Expr {
	pub fn new(layout: crate::Ref<Definition>, args: Vec<Self>) -> Self {
		Self {
			layout,
			args
		}
	}

	pub fn layout(&self) -> crate::Ref<Definition> {
		self.layout
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
		let layout_def = self.context().layouts().get(self.expr().layout).unwrap();
		let iri = self.context().vocabulary().get(layout_def.id).unwrap();

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