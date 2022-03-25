use crate::{layout, ty, Causes, Documentation, Id, MaybeSet, WithCauses};
use shelves::Ref;

mod enumeration;
mod literal;
mod native;
mod structure;
mod sum;

mod strongly_connected;
mod usages;

pub use enumeration::{Enum, Variant};
pub use literal::Literal;
pub use native::Native;
pub use structure::{Field, Struct};
pub use sum::Sum;

pub use strongly_connected::StronglyConnectedLayouts;
pub use usages::Usages;

/// Layout type.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Type {
	Native(Native),
	Struct,
	Enum,
	Reference,
	Sum,
	Literal,
}

/// Layout definition.
pub struct Definition<F> {
	id: Id,
	ty: WithCauses<Ref<ty::Definition<F>>, F>,
	desc: WithCauses<Description<F>, F>,
	causes: Causes<F>,
}

/// Layout description.
pub enum Description<F> {
	/// Native layout, such as a number, a string, etc.
	Native(Native, MaybeSet<String, F>),

	/// Structure.
	Struct(Struct<F>),

	/// Enumeration.
	Enum(Enum<F>),

	/// Reference.
	Reference(Ref<layout::Definition<F>>, MaybeSet<String, F>),

	/// Sum type.
	Sum(Sum<F>),

	/// Literal layout.
	Literal(Literal<F>),
}

impl<F> Description<F> {
	pub fn ty(&self) -> Type {
		match self {
			Self::Reference(_, _) => Type::Reference,
			Self::Struct(_) => Type::Struct,
			Self::Enum(_) => Type::Enum,
			Self::Native(n, _) => Type::Native(*n),
			Self::Sum(_) => Type::Sum,
			Self::Literal(_) => Type::Literal,
		}
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
			Description::Enum(e) => Some(e.name()),
			Description::Reference(_, n) => n.value().map(String::as_str),
			Description::Native(_, n) => n.value().map(String::as_str),
			Description::Sum(s) => Some(s.name()),
			Description::Literal(l) => Some(l.name()),
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
			Description::Enum(e) => ComposingLayouts::Enum(e.fields()),
			Description::Sum(s) => ComposingLayouts::Sum(s.options().iter()),
			Description::Literal(_) => ComposingLayouts::None,
			Description::Reference(_, _) => ComposingLayouts::None,
			Description::Native(_, _) => ComposingLayouts::None,
		}
	}
}

pub enum ComposingLayouts<'a, F> {
	Struct(std::slice::Iter<'a, Field<F>>),
	Enum(enumeration::Fields<'a, F>),
	Sum(std::slice::Iter<'a, WithCauses<Ref<Definition<F>>, F>>),
	None,
}

impl<'a, F> Iterator for ComposingLayouts<'a, F> {
	type Item = Ref<Definition<F>>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Struct(fields) => Some(fields.next()?.layout()),
			Self::Enum(fields) => Some(fields.next()?.layout()),
			Self::Sum(layouts) => layouts.next().map(WithCauses::inner).cloned(),
			Self::None => None,
		}
	}
}
