use crate::{layout, prop, ty, Cause, Caused, Causes, Documentation, Error, Id, Ref, WithCauses};
use std::fmt;

mod strongly_connected;
mod usages;

pub use strongly_connected::StronglyConnectedLayouts;
pub use usages::Usages;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Type {
	Struct,
	Native(Native),
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
	Url,
	Reference(Ref<layout::Definition>),
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
	Native(Native),
}

impl Description {
	pub fn ty(&self) -> Type {
		match self {
			Self::Struct(_) => Type::Struct,
			Self::Native(n) => Type::Native(*n),
		}
	}
}

impl WithCauses<Description> {
	pub fn check(
		&self,
		model: &crate::Model,
		ty: Ref<ty::Definition>,
	) -> Result<(), Caused<Error>> {
		match self.inner() {
			Description::Native(_) => Ok(()),
			Description::Struct(fields) => fields.check(model, self.causes(), ty),
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
			desc: None,
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
				None => &self.doc,
			}
		} else {
			&self.doc
		}
	}

	/// Declare the type for which this layout is defined.
	pub fn declare_type(
		&mut self,
		ty_ref: Ref<ty::Definition>,
		cause: Option<Cause>,
	) -> Result<(), Caused<Error>> {
		match &self.ty {
			Some(expected_ty) => {
				if *expected_ty.inner() != ty_ref {
					return Err(Caused::new(
						Error::LayoutTypeMismatch {
							expected: *expected_ty.inner(),
							found: ty_ref,
							because: expected_ty.causes().preferred(),
						},
						cause,
					));
				}
			}
			None => {
				self.ty = Some(WithCauses::new(ty_ref, cause));
			}
		}

		Ok(())
	}

	pub fn declare_native(
		&mut self,
		native: Native,
		cause: Option<Cause>,
	) -> Result<(), Caused<Mismatch>> {
		match &mut self.desc {
			Some(desc) => match desc.inner_mut() {
				Description::Native(n) if *n == native => Ok(()),
				_ => Err(Caused::new(
					Mismatch::Type {
						expected: desc.ty(),
						found: Type::Struct,
						because: desc.causes().preferred(),
					},
					cause,
				)),
			},
			None => {
				self.desc = Some(WithCauses::new(Description::Native(native), cause));
				Ok(())
			}
		}
	}

	pub fn declare_fields(
		&mut self,
		fields: Vec<Field>,
		cause: Option<Cause>,
	) -> Result<(), Caused<Mismatch>> {
		match &mut self.desc {
			Some(desc) => {
				let desc_cause = desc.causes().preferred();
				match desc.inner_mut() {
					Description::Struct(current_fields) => {
						current_fields.add_causes(desc_cause, &fields, cause)
					}
					_ => Err(Caused::new(
						Mismatch::Type {
							expected: desc.ty(),
							found: Type::Struct,
							because: desc.causes().preferred(),
						},
						cause,
					)),
				}
			}
			None => {
				self.desc = Some(WithCauses::new(
					Description::Struct(Fields::new(fields)),
					cause,
				));
				Ok(())
			}
		}
	}

	pub fn set_fields(&mut self, fields: Vec<Field>, causes: impl Into<Causes>) {
		self.desc = Some(WithCauses::new(
			Description::Struct(Fields::new(fields)),
			causes,
		))
	}

	pub fn check(&self, model: &crate::Model) -> Result<(), Caused<Error>> {
		let ty = *self.ty().expect("undefined layout").inner();

		if let Some(desc) = &self.desc {
			desc.check(model, ty)?
		}

		Ok(())
	}

	pub fn composing_layouts(&self) -> Option<ComposingLayouts> {
		match self.description()?.inner() {
			Description::Struct(fields) => Some(ComposingLayouts::Struct(fields.iter())),
			Description::Native(_) => Some(ComposingLayouts::Native),
		}
	}
}

pub enum ComposingLayouts<'a> {
	Struct(std::slice::Iter<'a, Field>),
	Native,
}

impl<'a> Iterator for ComposingLayouts<'a> {
	type Item = Ref<Definition>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Struct(fields) => Some(fields.next()?.layout()),
			Self::Native => None,
		}
	}
}

/// Layout mismatch error.
#[derive(Debug)]
pub enum Mismatch {
	Type {
		expected: Type,
		found: Type,
		because: Option<Cause>,
	},
	FieldProperty {
		expected: Ref<prop::Definition>,
		found: Ref<prop::Definition>,
		because: Option<Cause>,
	},
	FieldName {
		expected: String,
		found: String,
		because: Option<Cause>,
	},
	FieldLayout {
		expected: Ref<Definition>,
		found: Ref<Definition>,
		because: Option<Cause>,
	},
	AttributeRequired {
		/// Is the field required?
		///
		/// If `true` then it is, and some other declaration is missing the `required` attribute.
		/// If `false` then it is not, and some other declaration is adding the attribute.
		required: bool,
		because: Option<Cause>,
	},
	AttributeFunctional {
		functional: bool,
		because: Option<Cause>,
	},
	MissingField {
		name: String,
		because: Option<Cause>,
	},
	AdditionalField {
		name: String,
		because: Option<Cause>,
	},
}

/// Layout fields.
pub struct Fields {
	fields: Vec<Field>,
}

impl Fields {
	pub fn new(fields: Vec<Field>) -> Self {
		Self { fields }
	}

	pub fn check(
		&self,
		model: &crate::Model,
		causes: &Causes,
		ty_ref: Ref<ty::Definition>,
	) -> Result<(), Caused<Error>> {
		let ty = model.types().get(ty_ref).unwrap();

		for (prop_ref, _) in ty.properties() {
			let prop = model.properties().get(prop_ref).unwrap();
			if prop.is_required() && !self.contains_prop(prop_ref) {
				return Err(Caused::new(
					Error::MissingPropertyField {
						prop: prop_ref,
						because: prop.causes().preferred(),
					},
					causes.preferred(),
				));
			}
		}

		for f in &self.fields {
			f.check(model)?
		}

		Ok(())
	}

	pub fn contains_prop(&self, prop_ref: Ref<prop::Definition>) -> bool {
		self.fields.iter().any(|f| f.prop == prop_ref)
	}

	pub fn as_slice(&self) -> &[Field] {
		&self.fields
	}

	pub fn iter(&self) -> std::slice::Iter<Field> {
		self.fields.iter()
	}

	pub fn add_causes(
		&mut self,
		self_cause: Option<Cause>,
		fields: &[Field],
		cause: Option<Cause>,
	) -> Result<(), Caused<Mismatch>> {
		for (a, b) in self.fields.iter().zip(fields) {
			if a.property() != b.property() {
				return Err(Caused::new(
					Mismatch::FieldProperty {
						expected: a.property(),
						found: b.property(),
						because: a.causes().preferred(),
					},
					b.causes().preferred(),
				));
			}

			if a.name() != b.name() {
				return Err(Caused::new(
					Mismatch::FieldName {
						expected: a.name().to_owned(),
						found: b.name().to_owned(),
						because: a.causes().preferred(),
					},
					b.causes().preferred(),
				));
			}

			if a.layout() != b.layout() {
				return Err(Caused::new(
					Mismatch::FieldLayout {
						expected: a.layout(),
						found: b.layout(),
						because: a.causes().preferred(),
					},
					b.causes().preferred(),
				));
			}

			if a.is_required() != b.is_required() {
				return Err(Caused::new(
					Mismatch::AttributeRequired {
						required: a.is_required(),
						because: a.causes().preferred(),
					},
					b.causes().preferred(),
				));
			}

			if a.is_functional() != b.is_functional() {
				return Err(Caused::new(
					Mismatch::AttributeFunctional {
						functional: a.is_functional(),
						because: a.causes().preferred(),
					},
					b.causes().preferred(),
				));
			}
		}

		if self.fields.len() > fields.len() {
			let field = &self.fields[fields.len()];
			return Err(Caused::new(
				Mismatch::MissingField {
					name: field.name().to_owned(),
					because: field.causes().preferred(),
				},
				cause,
			));
		}

		if fields.len() > self.fields.len() {
			let field = &fields[self.fields.len()];
			return Err(Caused::new(
				Mismatch::AdditionalField {
					name: field.name().to_owned(),
					because: self_cause,
				},
				field.causes().preferred(),
			));
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
	layout: Ref<Definition>,
	required: bool,
	functional: bool,
	causes: Causes,
	doc: Documentation,
}

impl Field {
	pub fn new(
		prop: Ref<prop::Definition>,
		name: String,
		layout: Ref<Definition>,
		causes: impl Into<Causes>,
	) -> Self {
		Self {
			prop,
			name,
			layout,
			required: false,
			functional: true,
			causes: causes.into(),
			doc: Documentation::default(),
		}
	}

	/// Check the well-formedness of this field.
	///
	/// The layout must be fit for the given property type.
	/// The field must be required if the property is required.
	pub fn check(&self, model: &crate::Model) -> Result<(), Caused<Error>> {
		let prop = model.properties().get(self.prop).unwrap();

		if prop.is_required() && !self.is_required() {
			return Err(Caused::new(
				Error::FieldNotRequired {
					prop: self.prop,
					because: prop.causes().preferred(),
				},
				self.causes().preferred(),
			));
		}

		if prop.is_functional() && !self.is_functional() {
			return Err(Caused::new(
				Error::FieldNotFunctional {
					prop: self.prop,
					because: prop.causes().preferred(),
				},
				self.causes().preferred(),
			));
		}

		Ok(())
	}

	pub fn property(&self) -> Ref<prop::Definition> {
		self.prop
	}

	pub fn name(&self) -> &str {
		&self.name
	}

	pub fn layout(&self) -> Ref<Definition> {
		self.layout
	}

	pub fn is_required(&self) -> bool {
		self.required
	}

	pub fn declare_required(&mut self) {
		self.required = true
	}

	pub fn set_required(&mut self, v: bool) {
		self.required = v
	}

	pub fn is_functional(&self) -> bool {
		self.functional
	}

	pub fn declare_functional(&mut self) {
		self.functional = true
	}

	pub fn declare_multiple(&mut self) {
		self.functional = false
	}

	pub fn set_functional(&mut self, v: bool) {
		self.functional = v
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

impl Ref<Definition> {
	pub fn with_model<'c>(&self, context: &'c crate::Model) -> RefWithContext<'c> {
		RefWithContext(context, *self)
	}
}

pub struct RefWithContext<'c>(&'c crate::Model, Ref<Definition>);

impl<'c> fmt::Display for RefWithContext<'c> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let id = self.0.layouts().get(self.1).unwrap().id();
		let iri = self.0.vocabulary().get(id).unwrap();
		iri.fmt(f)
	}
}

// #[derive(Clone, PartialEq, Eq, Debug)]
// pub struct Expr {
// 	layout: crate::Ref<Definition>,
// 	args: Vec<Self>,
// }

// impl Expr {
// 	pub fn new(layout: crate::Ref<Definition>, args: Vec<Self>) -> Self {
// 		Self { layout, args }
// 	}

// 	pub fn layout(&self) -> crate::Ref<Definition> {
// 		self.layout
// 	}

// 	pub fn arguments(&self) -> &[Self] {
// 		&self.args
// 	}

// 	pub fn with_model<'c>(&self, context: &'c crate::Model) -> ExprWithContext<'c, '_> {
// 		ExprWithContext(context, self)
// 	}
// }

// pub struct ExprWithContext<'c, 'e>(&'c crate::Model, &'e Expr);

// impl<'c, 'e> ExprWithContext<'c, 'e> {
// 	pub fn context(&self) -> &'c crate::Model {
// 		self.0
// 	}

// 	pub fn expr(&self) -> &'e Expr {
// 		self.1
// 	}
// }

// impl<'c, 'e> fmt::Display for ExprWithContext<'c, 'e> {
// 	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
// 		let layout_def = self.context().layouts().get(self.expr().layout).unwrap();
// 		let iri = self.context().vocabulary().get(layout_def.id).unwrap();

// 		iri.fmt(f)?;

// 		if !self.expr().args.is_empty() {
// 			write!(f, "(")?;
// 			for (i, arg) in self.expr().args.iter().enumerate() {
// 				if i > 0 {
// 					write!(f, ", ")?;
// 				}

// 				arg.with_model(self.context()).fmt(f)?;
// 			}
// 			write!(f, ")")?;
// 		}

// 		Ok(())
// 	}
// }
