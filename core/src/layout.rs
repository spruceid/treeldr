use derivative::Derivative;
use crate::{layout, prop, ty, Caused, Causes, Documentation, Error, Id, Ref, WithCauses};
use std::fmt;
use locspan::Location;

mod strongly_connected;
mod usages;

pub use strongly_connected::StronglyConnectedLayouts;
pub use usages::Usages;

#[derive(Derivative)]
#[derivative(Clone(bound=""), Copy(bound=""), PartialEq(bound=""), Eq(bound=""), Hash(bound=""), Debug(bound=""))]
pub enum Type<F> {
	Struct,
	Native(Native<F>),
}

#[derive(Derivative)]
#[derivative(Clone(bound=""), Copy(bound=""), PartialEq(bound=""), Eq(bound=""), Hash(bound=""), Debug(bound=""))]
pub enum Native<F> {
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
	Reference(Ref<layout::Definition<F>>),
}

/// Layout definition.
pub struct Definition<F> {
	id: Id,
	ty: Option<WithCauses<Ref<ty::Definition<F>>, F>>,
	causes: Causes<F>,
	doc: Documentation,
	desc: Option<WithCauses<Description<F>, F>>,
}

pub enum Description<F> {
	Struct(Fields<F>),
	Native(Native<F>),
}

impl<F> Description<F> {
	pub fn ty(&self) -> Type<F> {
		match self {
			Self::Struct(_) => Type::Struct,
			Self::Native(n) => Type::Native(*n),
		}
	}
}

impl<F> WithCauses<Description<F>, F> {
	pub fn check(
		&self,
		model: &crate::Model<F>,
		ty: Ref<ty::Definition<F>>,
	) -> Result<(), Caused<Error<F>, F>> where F: Clone {
		match self.inner() {
			Description::Native(_) => Ok(()),
			Description::Struct(fields) => fields.check(model, self.causes(), ty),
		}
	}
}

impl<F> Definition<F> {
	pub fn new(id: Id, causes: impl Into<Causes<F>>) -> Self {
		Self {
			id,
			ty: None,
			causes: causes.into(),
			doc: Documentation::default(),
			desc: None,
		}
	}

	/// Type for which the layout is defined.
	pub fn ty(&self) -> Option<&WithCauses<Ref<ty::Definition<F>>, F>> {
		self.ty.as_ref()
	}

	/// Returns the identifier of the defined layout.
	pub fn id(&self) -> Id {
		self.id
	}

	pub fn causes(&self) -> &Causes<F> {
		&self.causes
	}

	pub fn description(&self) -> Option<&WithCauses<Description<F>, F>> {
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

	pub fn preferred_documentation<'a>(&'a self, model: &'a crate::Model<F>) -> &'a Documentation {
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
		ty_ref: Ref<ty::Definition<F>>,
		cause: Option<Location<F>>,
	) -> Result<(), Caused<Error<F>, F>> where F: Clone + Ord {
		match &self.ty {
			Some(expected_ty) => {
				if *expected_ty.inner() != ty_ref {
					return Err(Caused::new(
						Error::LayoutTypeMismatch {
							expected: *expected_ty.inner(),
							found: ty_ref,
							because: expected_ty.causes().preferred().cloned(),
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
		native: Native<F>,
		cause: Option<Location<F>>,
	) -> Result<(), Caused<Mismatch<F>, F>> where F: Clone + Ord {
		match &mut self.desc {
			Some(desc) => match desc.inner_mut() {
				Description::Native(n) if *n == native => Ok(()),
				_ => Err(Caused::new(
					Mismatch::Type {
						expected: desc.ty(),
						found: Type::Struct,
						because: desc.causes().preferred().cloned(),
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
		fields: Vec<Field<F>>,
		cause: Option<Location<F>>,
	) -> Result<(), Caused<Mismatch<F>, F>> where F: Clone + Ord {
		match &mut self.desc {
			Some(desc) => {
				let desc_cause = desc.causes().preferred().cloned();
				match desc.inner_mut() {
					Description::Struct(current_fields) => {
						current_fields.add_causes(desc_cause, &fields, cause)
					}
					_ => Err(Caused::new(
						Mismatch::Type {
							expected: desc.ty(),
							found: Type::Struct,
							because: desc.causes().preferred().cloned(),
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

	pub fn set_fields(&mut self, fields: Vec<Field<F>>, causes: impl Into<Causes<F>>) {
		self.desc = Some(WithCauses::new(
			Description::Struct(Fields::new(fields)),
			causes,
		))
	}

	pub fn check(&self, model: &crate::Model<F>) -> Result<(), Caused<Error<F>, F>> where F: Clone {
		let ty = *self.ty().expect("undefined layout").inner();

		if let Some(desc) = &self.desc {
			desc.check(model, ty)?
		}

		Ok(())
	}

	pub fn composing_layouts(&self) -> Option<ComposingLayouts<F>> {
		match self.description()?.inner() {
			Description::Struct(fields) => Some(ComposingLayouts::Struct(fields.iter())),
			Description::Native(_) => Some(ComposingLayouts::Native),
		}
	}
}

pub enum ComposingLayouts<'a, F> {
	Struct(std::slice::Iter<'a, Field<F>>),
	Native,
}

impl<'a, F> Iterator for ComposingLayouts<'a, F> {
	type Item = Ref<Definition<F>>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Struct(fields) => Some(fields.next()?.layout()),
			Self::Native => None,
		}
	}
}

/// Layout mismatch error.
#[derive(Debug)]
pub enum Mismatch<F> {
	Type {
		expected: Type<F>,
		found: Type<F>,
		because: Option<Location<F>>,
	},
	FieldProperty {
		expected: Ref<prop::Definition<F>>,
		found: Ref<prop::Definition<F>>,
		because: Option<Location<F>>,
	},
	FieldName {
		expected: String,
		found: String,
		because: Option<Location<F>>,
	},
	FieldLayout {
		expected: Ref<Definition<F>>,
		found: Ref<Definition<F>>,
		because: Option<Location<F>>,
	},
	AttributeRequired {
		/// Is the field required?
		///
		/// If `true` then it is, and some other declaration is missing the `required` attribute.
		/// If `false` then it is not, and some other declaration is adding the attribute.
		required: bool,
		because: Option<Location<F>>,
	},
	AttributeFunctional {
		functional: bool,
		because: Option<Location<F>>,
	},
	MissingField {
		name: String,
		because: Option<Location<F>>,
	},
	AdditionalField {
		name: String,
		because: Option<Location<F>>,
	},
}

/// Layout fields.
pub struct Fields<F> {
	fields: Vec<Field<F>>,
}

impl<F> Fields<F> {
	pub fn new(fields: Vec<Field<F>>) -> Self {
		Self { fields }
	}

	pub fn check(
		&self,
		model: &crate::Model<F>,
		causes: &Causes<F>,
		ty_ref: Ref<ty::Definition<F>>,
	) -> Result<(), Caused<Error<F>, F>> where F: Clone {
		let ty = model.types().get(ty_ref).unwrap();

		for (prop_ref, _) in ty.properties() {
			let prop = model.properties().get(prop_ref).unwrap();
			if prop.is_required() && !self.contains_prop(prop_ref) {
				return Err(Caused::new(
					Error::MissingPropertyField {
						prop: prop_ref,
						because: prop.causes().preferred().cloned(),
					},
					causes.preferred().cloned(),
				));
			}
		}

		for f in &self.fields {
			f.check(model)?
		}

		Ok(())
	}

	pub fn contains_prop(&self, prop_ref: Ref<prop::Definition<F>>) -> bool {
		self.fields.iter().any(|f| f.prop == prop_ref)
	}

	pub fn as_slice(&self) -> &[Field<F>] {
		&self.fields
	}

	pub fn iter(&self) -> std::slice::Iter<Field<F>> {
		self.fields.iter()
	}

	pub fn add_causes(
		&mut self,
		self_cause: Option<Location<F>>,
		fields: &[Field<F>],
		cause: Option<Location<F>>,
	) -> Result<(), Caused<Mismatch<F>, F>> where F: Clone {
		for (a, b) in self.fields.iter().zip(fields) {
			if a.property() != b.property() {
				return Err(Caused::new(
					Mismatch::FieldProperty {
						expected: a.property(),
						found: b.property(),
						because: a.causes().preferred().cloned(),
					},
					b.causes().preferred().cloned(),
				));
			}

			if a.name() != b.name() {
				return Err(Caused::new(
					Mismatch::FieldName {
						expected: a.name().to_owned(),
						found: b.name().to_owned(),
						because: a.causes().preferred().cloned(),
					},
					b.causes().preferred().cloned(),
				));
			}

			if a.layout() != b.layout() {
				return Err(Caused::new(
					Mismatch::FieldLayout {
						expected: a.layout(),
						found: b.layout(),
						because: a.causes().preferred().cloned(),
					},
					b.causes().preferred().cloned(),
				));
			}

			if a.is_required() != b.is_required() {
				return Err(Caused::new(
					Mismatch::AttributeRequired {
						required: a.is_required(),
						because: a.causes().preferred().cloned(),
					},
					b.causes().preferred().cloned(),
				));
			}

			if a.is_functional() != b.is_functional() {
				return Err(Caused::new(
					Mismatch::AttributeFunctional {
						functional: a.is_functional(),
						because: a.causes().preferred().cloned(),
					},
					b.causes().preferred().cloned(),
				));
			}
		}

		if self.fields.len() > fields.len() {
			let field = &self.fields[fields.len()];
			return Err(Caused::new(
				Mismatch::MissingField {
					name: field.name().to_owned(),
					because: field.causes().preferred().cloned(),
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
				field.causes().preferred().cloned(),
			));
		}

		Ok(())
	}
}

impl<F> AsRef<[Field<F>]> for Fields<F> {
	fn as_ref(&self) -> &[Field<F>] {
		self.as_slice()
	}
}

impl<F> std::ops::Deref for Fields<F> {
	type Target = [Field<F>];

	fn deref(&self) -> &[Field<F>] {
		self.as_slice()
	}
}

impl<'a, F> IntoIterator for &'a Fields<F> {
	type Item = &'a Field<F>;
	type IntoIter = std::slice::Iter<'a, Field<F>>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

/// Layout field.
pub struct Field<F> {
	prop: Ref<prop::Definition<F>>,
	name: String,
	layout: Ref<Definition<F>>,
	required: bool,
	functional: bool,
	causes: Causes<F>,
	doc: Documentation,
}

impl<F> Field<F> {
	pub fn new(
		prop: Ref<prop::Definition<F>>,
		name: String,
		layout: Ref<Definition<F>>,
		causes: impl Into<Causes<F>>,
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
	pub fn check(&self, model: &crate::Model<F>) -> Result<(), Caused<Error<F>, F>> where F: Clone {
		let prop = model.properties().get(self.prop).unwrap();

		if prop.is_required() && !self.is_required() {
			return Err(Caused::new(
				Error::FieldNotRequired {
					prop: self.prop,
					because: prop.causes().preferred().cloned(),
				},
				self.causes().preferred().cloned(),
			));
		}

		if prop.is_functional() && !self.is_functional() {
			return Err(Caused::new(
				Error::FieldNotFunctional {
					prop: self.prop,
					because: prop.causes().preferred().cloned(),
				},
				self.causes().preferred().cloned(),
			));
		}

		Ok(())
	}

	pub fn property(&self) -> Ref<prop::Definition<F>> {
		self.prop
	}

	pub fn name(&self) -> &str {
		&self.name
	}

	pub fn layout(&self) -> Ref<Definition<F>> {
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

	pub fn preferred_documentation<'a>(&'a self, model: &'a crate::Model<F>) -> &'a Documentation {
		if self.doc.is_empty() {
			model.properties().get(self.prop).unwrap().documentation()
		} else {
			&self.doc
		}
	}
}

impl<F> Ref<Definition<F>> {
	pub fn with_model<'c>(&self, context: &'c crate::Model<F>) -> RefWithContext<'c, F> {
		RefWithContext(context, *self)
	}
}

pub struct RefWithContext<'c, F>(&'c crate::Model<F>, Ref<Definition<F>>);

impl<'c, F> fmt::Display for RefWithContext<'c, F> {
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
