use crate::{layout, prop, ty, Causes, Documentation, Id, MaybeSet, WithCauses};
use shelves::Ref;

mod strongly_connected;
mod usages;

pub use strongly_connected::StronglyConnectedLayouts;
pub use usages::Usages;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Type {
	Native(Native),
	Struct,
	Reference,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
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
}

/// Layout definition.
pub struct Definition<F> {
	id: Id,
	ty: WithCauses<Ref<ty::Definition<F>>, F>,
	desc: WithCauses<Description<F>, F>,
	causes: Causes<F>,
}

pub enum Description<F> {
	Native(Native, MaybeSet<String, F>),
	Struct(Struct<F>),
	Reference(Ref<layout::Definition<F>>, MaybeSet<String, F>),
}

impl<F> Description<F> {
	pub fn ty(&self) -> Type {
		match self {
			Self::Reference(_, _) => Type::Reference,
			Self::Struct(_) => Type::Struct,
			Self::Native(n, _) => Type::Native(*n),
		}
	}

	pub fn is_native(&self) -> bool {
		matches!(self, Self::Native(_, _))
	}

	pub fn is_struct(&self) -> bool {
		matches!(self, Self::Struct(_))
	}

	pub fn is_reference(&self) -> bool {
		matches!(self, Self::Reference(_, _))
	}
}

impl<F> Definition<F> {
	pub fn new(
		id: Id,
		ty: WithCauses<Ref<ty::Definition<F>>, F>,
		desc: WithCauses<Description<F>, F>,
		causes: impl Into<Causes<F>>,
	) -> Self {
		Self {
			id,
			ty,
			desc,
			causes: causes.into(),
		}
	}

	/// Type for which the layout is defined.
	pub fn ty(&self) -> Ref<ty::Definition<F>> {
		*self.ty
	}

	/// Returns the identifier of the defined layout.
	pub fn id(&self) -> Id {
		self.id
	}

	pub fn name(&self) -> Option<&str> {
		match self.desc.inner() {
			Description::Struct(s) => Some(s.name()),
			Description::Reference(_, n) => n.value().map(String::as_str),
			Description::Native(_, n) => n.value().map(String::as_str),
		}
	}

	pub fn causes(&self) -> &Causes<F> {
		&self.causes
	}

	pub fn description(&self) -> &Description<F> {
		&self.desc
	}

	pub fn label<'m>(&self, model: &'m crate::Model<F>) -> Option<&'m str> {
		model.get(self.id).unwrap().label()
	}

	pub fn preferred_label<'a>(&'a self, model: &'a crate::Model<F>) -> Option<&'a str> {
		let label = self.label(model);
		if label.is_none() {
			let ty_id = model.types().get(*self.ty).unwrap().id();
			model.get(ty_id).unwrap().label()
		} else {
			label
		}
	}

	pub fn documentation<'m>(&self, model: &'m crate::Model<F>) -> &'m Documentation {
		model.get(self.id).unwrap().documentation()
	}

	pub fn preferred_documentation<'m>(&self, model: &'m crate::Model<F>) -> &'m Documentation {
		let doc = self.documentation(model);
		if doc.is_empty() {
			let ty_id = model.types().get(*self.ty).unwrap().id();
			model.get(ty_id).unwrap().documentation()
		} else {
			doc
		}
	}

	pub fn composing_layouts(&self) -> ComposingLayouts<F> {
		match self.description() {
			Description::Struct(s) => ComposingLayouts::Struct(s.fields().iter()),
			Description::Reference(_, _) => ComposingLayouts::Reference,
			Description::Native(_, _) => ComposingLayouts::Native,
		}
	}
}

pub enum ComposingLayouts<'a, F> {
	Struct(std::slice::Iter<'a, Field<F>>),
	Reference,
	Native,
}

impl<'a, F> Iterator for ComposingLayouts<'a, F> {
	type Item = Ref<Definition<F>>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Struct(fields) => Some(fields.next()?.layout()),
			Self::Reference => None,
			Self::Native => None,
		}
	}
}

/// Structure layout.
pub struct Struct<F> {
	name: WithCauses<String, F>,
	fields: Vec<Field<F>>,
}

impl<F> Struct<F> {
	pub fn new(name: WithCauses<String, F>, fields: Vec<Field<F>>) -> Self {
		Self { name, fields }
	}

	pub fn name(&self) -> &str {
		self.name.as_str()
	}

	pub fn fields(&self) -> &[Field<F>] {
		&self.fields
	}
}

/// Layout field.
pub struct Field<F> {
	prop: WithCauses<Ref<prop::Definition<F>>, F>,
	name: WithCauses<String, F>,
	label: Option<String>,
	layout: AnnotatedRef<F>,
	doc: Documentation,
}

impl<F> Field<F> {
	pub fn new(
		prop: WithCauses<Ref<prop::Definition<F>>, F>,
		name: WithCauses<String, F>,
		label: Option<String>,
		layout: WithCauses<Ref<Definition<F>>, F>,
		required: WithCauses<bool, F>,
		functional: WithCauses<bool, F>,
		doc: Documentation,
	) -> Self {
		Self {
			prop,
			name,
			label,
			layout: AnnotatedRef {
				layout,
				required,
				functional,
			},
			doc,
		}
	}

	pub fn property(&self) -> Ref<prop::Definition<F>> {
		*self.prop.inner()
	}

	pub fn name(&self) -> &str {
		self.name.as_str()
	}

	pub fn annotated_layout(&self) -> &AnnotatedRef<F> {
		&self.layout
	}

	pub fn label(&self) -> Option<&str> {
		self.label.as_deref()
	}

	pub fn preferred_label<'a>(&'a self, model: &'a crate::Model<F>) -> Option<&'a str> {
		if self.label.is_none() {
			let prop_id = model.properties().get(*self.prop).unwrap().id();
			model.get(prop_id).unwrap().label()
		} else {
			self.label.as_deref()
		}
	}

	pub fn layout(&self) -> Ref<layout::Definition<F>> {
		*self.layout.layout
	}

	pub fn is_required(&self) -> bool {
		*self.layout.required
	}

	pub fn is_functional(&self) -> bool {
		*self.layout.functional
	}

	pub fn documentation(&self) -> &Documentation {
		&self.doc
	}

	pub fn preferred_documentation<'a>(&'a self, model: &'a crate::Model<F>) -> &'a Documentation {
		if self.doc.is_empty() {
			let prop_id = model.properties().get(*self.prop).unwrap().id();
			model.get(prop_id).unwrap().documentation()
		} else {
			&self.doc
		}
	}
}

pub struct AnnotatedRef<F> {
	required: WithCauses<bool, F>,
	functional: WithCauses<bool, F>,
	layout: WithCauses<Ref<Definition<F>>, F>,
}

impl<F> AnnotatedRef<F> {
	pub fn is_required(&self) -> bool {
		*self.required
	}

	pub fn is_functional(&self) -> bool {
		*self.functional
	}

	pub fn layout(&self) -> Ref<Definition<F>> {
		*self.layout
	}
}
