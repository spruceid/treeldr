use crate::{
	error, layout, ty, vocab::Name, Caused, Causes, Documentation, Error, Id, MaybeSet, WithCauses,
};
use locspan::Location;
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
}

/// Layout definition.
pub struct Definition<F> {
	id: Id,
	ty: WithCauses<Ref<ty::Definition<F>>, F>,
	desc: WithCauses<Description<F>, F>,
	causes: Causes<F>,
}

/// Layout description.
#[derive(Clone)]
pub enum Description<F> {
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
}

impl<F> Description<F> {
	pub fn ty(&self) -> Type {
		match self {
			Self::Native(n, _) => Type::Native(*n),
			Self::Literal(_) => Type::Literal,
			Self::Reference(_, _) => Type::Reference,
			Self::Struct(_) => Type::Struct,
			Self::Enum(_) => Type::Enum,
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
			Self::Native(_, name) => name.replace(new_name, cause),
			Self::Literal(lit) => Some(lit.set_name(new_name, cause)),
			Self::Reference(_, name) => name.replace(new_name, cause),
			Self::Struct(s) => Some(s.set_name(new_name, cause)),
			Self::Enum(e) => Some(e.set_name(new_name, cause)),
		}
	}

	/// Intersects this type description with `other`.
	///
	/// If provided, `name` will override the name of the intersected type,
	/// otherwise the name of `self` is used.
	pub fn intersected_with(
		self,
		id: Id,
		other: &WithCauses<Self, F>,
		name: MaybeSet<Name, F>,
		built_layouts: &[Option<Definition<F>>],
	) -> Result<Self, Error<F>>
	where
		F: Clone + Ord,
	{
		match (self, other.inner()) {
			(Self::Native(a, a_name), Self::Native(b, _)) if &a == b => {
				Ok(Self::Native(a, name.or(a_name)))
			}
			(Self::Reference(a, a_name), Self::Reference(b, _)) if &a == b => {
				Ok(Self::Reference(a, name.or(a_name)))
			}
			(Self::Literal(a), Self::Literal(b)) => Ok(Self::Literal(a.intersected_with(
				id,
				b,
				name,
				other.causes().preferred(),
			)?)),
			(Self::Struct(a), Self::Struct(b)) => Ok(Self::Struct(a.intersected_with(
				id,
				b,
				name,
				other.causes().preferred(),
			)?)),
			(Self::Enum(a), Self::Enum(b)) => {
				let e = a.intersected_with(id, b, name, other.causes().preferred())?;

				if e.variants().len() == 1 && e.variants()[0].layout().is_some() {
					let layout_ref = e.variants()[0].layout().unwrap();
					let mut desc = built_layouts[layout_ref.index()]
						.as_ref()
						.unwrap()
						.description()
						.clone();
					desc.set_name(e.name().clone(), e.name_causes().preferred().cloned());
					Ok(desc)
				} else {
					Ok(Self::Enum(e))
				}
			}
			_ => Err(Caused::new(
				error::LayoutIntersectionFailed { id }.into(),
				other.causes().preferred().cloned(),
			)),
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

	pub fn name(&self) -> Option<&Name> {
		match self.desc.inner() {
			Description::Struct(s) => Some(s.name()),
			Description::Enum(e) => Some(e.name()),
			Description::Reference(_, n) => n.value(),
			Description::Native(_, n) => n.value(),
			Description::Literal(l) => Some(l.name()),
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
			Description::Enum(e) => ComposingLayouts::Enum(e.composing_layouts()),
			Description::Literal(_) => ComposingLayouts::None,
			Description::Reference(_, _) => ComposingLayouts::None,
			Description::Native(_, _) => ComposingLayouts::None,
		}
	}
}

pub enum ComposingLayouts<'a, F> {
	Struct(std::slice::Iter<'a, Field<F>>),
	Enum(enumeration::ComposingLayouts<'a, F>),
	None,
}

impl<'a, F> Iterator for ComposingLayouts<'a, F> {
	type Item = Ref<Definition<F>>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Struct(fields) => Some(fields.next()?.layout()),
			Self::Enum(layouts) => layouts.next(),
			Self::None => None,
		}
	}
}
