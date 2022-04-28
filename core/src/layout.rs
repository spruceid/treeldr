use crate::{layout, ty, vocab::Name, Causes, Documentation, Id, MaybeSet, WithCauses};
use locspan::Location;
use shelves::Ref;

mod array;
pub mod enumeration;
pub mod literal;
mod native;
mod set;
mod structure;

mod strongly_connected;
mod usages;

pub use array::Array;
pub use enumeration::{Enum, Variant};
pub use literal::Literal;
pub use native::Native;
pub use set::Set;
pub use structure::{Field, Struct};

pub use strongly_connected::StronglyConnectedLayouts;
pub use usages::Usages;

/// Layout type.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Type {
	Never,
	Native(Native),
	Struct,
	Enum,
	Reference,
	Literal,
	Array,
	Set,
}

/// Layout definition.
pub struct Definition<F> {
	id: Id,
	ty: MaybeSet<Ref<ty::Definition<F>>, F>,
	desc: WithCauses<Description<F>, F>,
	causes: Causes<F>,
}

/// Layout description.
#[derive(Clone)]
pub enum Description<F> {
	/// Never layout.
	Never(MaybeSet<Name, F>),

	/// Native layout, such as a number, a string, etc.
	Native(Native, MaybeSet<Name, F>),

	/// Reference.
	Reference(Ref<layout::Definition<F>>, MaybeSet<Name, F>),

	/// Literal layout.
	Literal(Literal<F>),

	/// Structure.
	Struct(Struct<F>),

	/// Enumeration.
	Enum(Enum<F>),

	/// Array layout.
	Array(Array<F>),

	/// Set layout.
	Set(Set<F>),
}

impl<F> Description<F> {
	pub fn ty(&self) -> Type {
		match self {
			Self::Never(_) => Type::Never,
			Self::Native(n, _) => Type::Native(*n),
			Self::Literal(_) => Type::Literal,
			Self::Reference(_, _) => Type::Reference,
			Self::Struct(_) => Type::Struct,
			Self::Enum(_) => Type::Enum,
			Self::Array(_) => Type::Array,
			Self::Set(_) => Type::Set,
		}
	}

	pub fn set_name(
		&mut self,
		new_name: Name,
		cause: Option<Location<F>>,
	) -> Option<WithCauses<Name, F>>
	where
		F: Ord,
	{
		match self {
			Self::Never(name) => name.replace(new_name, cause),
			Self::Native(_, name) => name.replace(new_name, cause),
			Self::Literal(lit) => Some(lit.set_name(new_name, cause)),
			Self::Reference(_, name) => name.replace(new_name, cause),
			Self::Struct(s) => Some(s.set_name(new_name, cause)),
			Self::Enum(e) => Some(e.set_name(new_name, cause)),
			Self::Array(a) => a.set_name(new_name, cause),
			Self::Set(s) => s.set_name(new_name, cause),
		}
	}

	pub fn into_name(self) -> MaybeSet<Name, F> {
		match self {
			Description::Never(n) => n,
			Description::Struct(s) => s.into_name().into(),
			Description::Enum(e) => e.into_name().into(),
			Description::Reference(_, n) => n,
			Description::Native(_, n) => n,
			Description::Literal(l) => l.into_name().into(),
			Description::Array(a) => a.into_name(),
			Description::Set(s) => s.into_name(),
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
			Description::Never(n) => n.value(),
			Description::Struct(s) => Some(s.name()),
			Description::Enum(e) => Some(e.name()),
			Description::Reference(_, n) => n.value(),
			Description::Native(_, n) => n.value(),
			Description::Literal(l) => Some(l.name()),
			Description::Array(a) => a.name(),
			Description::Set(s) => s.name(),
		}
	}

	pub fn causes(&self) -> &Causes<F> {
		&self.causes
	}

	pub fn description(&self) -> &Description<F> {
		&self.desc
	}

	pub fn description_with_causes(&self) -> &WithCauses<Description<F>, F> {
		&self.desc
	}

	pub fn label<'m>(&self, model: &'m crate::Model<F>) -> Option<&'m str> {
		model.get(self.id).unwrap().label()
	}

	pub fn preferred_label<'a>(&'a self, model: &'a crate::Model<F>) -> Option<&'a str> {
		let label = self.label(model);
		if label.is_none() {
			self.ty().and_then(|ty_ref| {
				let ty_id = model.types().get(ty_ref).unwrap().id();
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
		if doc.is_empty() && self.ty().is_some() {
			let ty_id = model.types().get(self.ty().unwrap()).unwrap().id();
			model.get(ty_id).unwrap().documentation()
		} else {
			doc
		}
	}

	pub fn composing_layouts(&self) -> ComposingLayouts<F> {
		match self.description() {
			Description::Never(_) => ComposingLayouts::None,
			Description::Struct(s) => ComposingLayouts::Struct(s.fields().iter()),
			Description::Enum(e) => ComposingLayouts::Enum(e.composing_layouts()),
			Description::Literal(_) => ComposingLayouts::None,
			Description::Reference(_, _) => ComposingLayouts::None,
			Description::Native(_, _) => ComposingLayouts::None,
			Description::Array(a) => ComposingLayouts::One(Some(a.item_layout())),
			Description::Set(s) => ComposingLayouts::One(Some(s.item_layout())),
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
