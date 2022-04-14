use crate::{layout, ty, vocab::Name, Causes, Documentation, Id, MaybeSet, WithCauses};
use shelves::Ref;

pub mod enumeration;
pub mod literal;
mod native;
mod structure;

mod strongly_connected;
mod usages;

pub use enumeration::{Enum, Variant};
pub use literal::Literal;
pub use native::Native;
pub use structure::{Field, Struct};

pub use strongly_connected::StronglyConnectedLayouts;
pub use usages::Usages;

/// Layout type.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Type {
	Native(Native),
	Struct,
	Enum,
	Reference,
	Literal,
	Set,
	List,
}

/// Layout definition.
pub struct Definition<F> {
	id: Id,

	/// Type for witch this layout is defined.
	///
	/// If unset, this layout is an "orphan" layout.
	ty: MaybeSet<Ref<ty::Definition<F>>, F>,
	desc: WithCauses<Description<F>, F>,
	causes: Causes<F>,
}

/// Layout description.
pub enum Description<F> {
	/// Native layout, such as a number, a string, etc.
	Native(Native, MaybeSet<Name, F>),

	/// Structure.
	Struct(Struct<F>),

	/// Enumeration.
	Enum(Enum<F>),

	/// Reference.
	Reference(Ref<layout::Definition<F>>, MaybeSet<Name, F>),

	/// Literal layout.
	Literal(Literal<F>),

	/// Set layout.
	Set(Ref<layout::Definition<F>>, MaybeSet<Name, F>),

	/// List/array layout.
	List(Ref<layout::Definition<F>>, MaybeSet<Name, F>),
}

impl<F> Description<F> {
	pub fn ty(&self) -> Type {
		match self {
			Self::Reference(_, _) => Type::Reference,
			Self::Struct(_) => Type::Struct,
			Self::Enum(_) => Type::Enum,
			Self::Native(n, _) => Type::Native(*n),
			Self::Literal(_) => Type::Literal,
			Self::Set(_, _) => Type::Set,
			Self::List(_, _) => Type::List,
		}
	}
}

impl<F> Definition<F> {
	pub fn new(
		id: Id,
		ty: MaybeSet<Ref<ty::Definition<F>>, F>,
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
	pub fn ty(&self) -> Option<Ref<ty::Definition<F>>> {
		self.ty.value().cloned()
	}

	/// Returns the identifier of the defined layout.
	pub fn id(&self) -> Id {
		self.id
	}

	pub fn name(&self) -> Option<&Name> {
		match self.desc.inner() {
			Description::Struct(s) => Some(s.name()),
			Description::Enum(e) => Some(e.name()),
			Description::Reference(_, n) => n.value(),
			Description::Native(_, n) => n.value(),
			Description::Literal(l) => Some(l.name()),
			Description::Set(_, n) => n.value(),
			Description::List(_, n) => n.value(),
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
			self.ty().and_then(|ty| {
				let ty_id = model.types().get(ty).unwrap().id();
				model.get(ty_id).unwrap().label()
			})
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
			match self.ty() {
				Some(ty) => {
					let ty_id = model.types().get(ty).unwrap().id();
					model.get(ty_id).unwrap().documentation()
				}
				None => doc,
			}
		} else {
			doc
		}
	}

	pub fn composing_layouts(&self) -> ComposingLayouts<F> {
		match self.description() {
			Description::Struct(s) => ComposingLayouts::Struct(s.fields().iter()),
			Description::Enum(e) => ComposingLayouts::Enum(e.composing_layouts()),
			Description::Literal(_) => ComposingLayouts::None,
			Description::Reference(_, _) => ComposingLayouts::None,
			Description::Native(_, _) => ComposingLayouts::None,
			Description::Set(i, _) => ComposingLayouts::One(Some(*i)),
			Description::List(i, _) => ComposingLayouts::One(Some(*i)),
		}
	}
}

pub enum ComposingLayouts<'a, F> {
	Struct(std::slice::Iter<'a, Field<F>>),
	Enum(enumeration::ComposingLayouts<'a, F>),
	One(Option<Ref<Definition<F>>>),
	None,
}

impl<'a, F> Iterator for ComposingLayouts<'a, F> {
	type Item = Ref<Definition<F>>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Struct(fields) => Some(fields.next()?.layout()),
			Self::Enum(layouts) => layouts.next(),
			Self::One(r) => r.take(),
			Self::None => None,
		}
	}
}
