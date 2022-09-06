use crate::{ty, Documentation, Id, MetaOption, Name};
use locspan::Meta;
use shelves::Ref;

pub mod array;
pub mod enumeration;
mod optional;
pub mod primitive;
mod reference;
mod required;
mod set;
mod structure;

mod strongly_connected;
mod usages;

pub use array::Array;
pub use enumeration::{Enum, Variant};
pub use optional::Optional;
pub use primitive::{restricted::Restricted as RestrictedPrimitive, Primitive};
pub use reference::Reference;
pub use required::Required;
pub use set::Set;
pub use structure::{Field, Struct};

pub use strongly_connected::StronglyConnectedLayouts;
pub use usages::Usages;

/// Layout kind.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Kind {
	Never,
	Primitive(Primitive),
	Struct,
	Enum,
	Reference,
	Literal,
	Required,
	Option,
	Array,
	Set,
	Alias,
}

/// Layout definition.
pub struct Definition<M> {
	/// Identifier of the layout.
	id: Id,

	/// Type represented by this layout.
	ty: MetaOption<Ref<ty::Definition<M>>, M>,

	/// Layout description.
	desc: Meta<Description<M>, M>,

	// Metadata associated to the definition.
	metadata: M,
}

/// Layout description.
#[derive(Clone)]
pub enum Description<M> {
	/// Never layout.
	Never(MetaOption<Name, M>),

	/// Primitive layout, such as a number, a string, etc.
	Primitive(RestrictedPrimitive, MetaOption<Name, M>),

	/// Reference.
	Reference(Reference<M>),

	/// Structure.
	Struct(Struct<M>),

	/// Enumeration.
	Enum(Enum<M>),

	/// Required.
	Required(Required<M>),

	/// Option.
	Option(Optional<M>),

	/// Array.
	Array(Array<M>),

	/// Set.
	Set(Set<M>),

	/// Alias.
	Alias(Meta<Name, M>, Ref<Definition<M>>),
}

impl<M> Description<M> {
	pub fn kind(&self) -> Kind {
		match self {
			Self::Never(_) => Kind::Never,
			Self::Primitive(n, _) => Kind::Primitive(n.primitive()),
			Self::Reference(_) => Kind::Reference,
			Self::Struct(_) => Kind::Struct,
			Self::Enum(_) => Kind::Enum,
			Self::Required(_) => Kind::Required,
			Self::Option(_) => Kind::Option,
			Self::Array(_) => Kind::Array,
			Self::Set(_) => Kind::Set,
			Self::Alias(_, _) => Kind::Alias,
		}
	}

	/// Checks if this layout is required.
	///
	/// This means either the `required` container or a non-empty `set`
	/// container.
	pub fn is_required(&self) -> bool {
		matches!(self, Self::Required(_)) // TODO checks for non-empty sets
	}

	/// Sets the new name of the layout.
	pub fn set_name(&mut self, new_name: Name, metadata: M) -> Option<Meta<Name, M>> {
		match self {
			Self::Never(name) => name.replace(new_name, metadata),
			Self::Primitive(_, name) => name.replace(new_name, metadata),
			Self::Reference(r) => r.set_name(new_name, metadata),
			Self::Struct(s) => Some(s.set_name(new_name, metadata)),
			Self::Enum(e) => Some(e.set_name(new_name, metadata)),
			Self::Required(r) => r.set_name(new_name, metadata),
			Self::Option(o) => o.set_name(new_name, metadata),
			Self::Array(a) => a.set_name(new_name, metadata),
			Self::Set(s) => s.set_name(new_name, metadata),
			Self::Alias(n, _) => Some(std::mem::replace(n, Meta(new_name, metadata))),
		}
	}

	pub fn into_name(self) -> MetaOption<Name, M> {
		match self {
			Description::Never(n) => n,
			Description::Struct(s) => s.into_name().into(),
			Description::Enum(e) => e.into_name().into(),
			Description::Reference(r) => r.into_name(),
			Description::Primitive(_, n) => n,
			Description::Required(r) => r.into_name(),
			Description::Option(o) => o.into_name(),
			Description::Array(a) => a.into_name(),
			Description::Set(s) => s.into_name(),
			Description::Alias(n, _) => n.into(),
		}
	}
}

impl<M> Definition<M> {
	/// Creates a new layout definition.
	pub fn new(
		id: Id,
		ty: MetaOption<Ref<ty::Definition<M>>, M>,
		desc: Meta<Description<M>, M>,
		metadata: M,
	) -> Self {
		Self {
			id,
			ty,
			desc,
			metadata,
		}
	}

	/// Type for which the layout is defined.
	pub fn ty(&self) -> Option<Ref<ty::Definition<M>>> {
		self.ty.value().cloned()
	}

	/// Returns the identifier of the defined layout.
	pub fn id(&self) -> Id {
		self.id
	}

	pub fn name(&self) -> Option<&Meta<Name, M>> {
		match self.desc.value() {
			Description::Never(n) => n.as_ref(),
			Description::Struct(s) => Some(s.name()),
			Description::Enum(e) => Some(e.name()),
			Description::Reference(r) => r.name(),
			Description::Primitive(_, n) => n.as_ref(),
			Description::Required(r) => r.name(),
			Description::Option(o) => o.name(),
			Description::Array(a) => a.name(),
			Description::Set(s) => s.name(),
			Description::Alias(n, _) => Some(n),
		}
	}

	/// Returns a reference to the metadata associated to this definition.
	pub fn metadata(&self) -> &M {
		&self.metadata
	}

	/// Returns the layout description.
	pub fn description(&self) -> &Meta<Description<M>, M> {
		&self.desc
	}

	/// Checks if the layout is required.
	///
	/// This means either the `required` container or a non-empty `set`
	/// container.
	pub fn is_required(&self) -> bool {
		self.desc.is_required()
	}

	/// Returns the defined label for this layout.
	pub fn label<'m>(&self, model: &'m crate::Model<M>) -> Option<&'m str> {
		model.get(self.id).unwrap().label()
	}

	/// Returns the preferred layout for this layout.
	///
	/// Either the defined label if any, or the type label otherwise (if any).
	pub fn preferred_label<'a>(&'a self, model: &'a crate::Model<M>) -> Option<&'a str> {
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

	pub fn documentation<'m>(&self, model: &'m crate::Model<M>) -> &'m Documentation {
		model.get(self.id).unwrap().documentation()
	}

	pub fn preferred_documentation<'m>(&self, model: &'m crate::Model<M>) -> &'m Documentation {
		let doc = self.documentation(model);
		if doc.is_empty() && self.ty().is_some() {
			let ty_id = model.types().get(self.ty().unwrap()).unwrap().id();
			model.get(ty_id).unwrap().documentation()
		} else {
			doc
		}
	}

	pub fn composing_layouts(&self) -> ComposingLayouts<M> {
		match self.description().value() {
			Description::Never(_) => ComposingLayouts::None,
			Description::Struct(s) => ComposingLayouts::Struct(s.fields().iter()),
			Description::Enum(e) => ComposingLayouts::Enum(e.composing_layouts()),
			Description::Reference(r) => ComposingLayouts::One(Some(r.id_layout())),
			Description::Primitive(_, _) => ComposingLayouts::None,
			Description::Option(o) => ComposingLayouts::One(Some(o.item_layout())),
			Description::Required(r) => ComposingLayouts::One(Some(r.item_layout())),
			Description::Array(a) => ComposingLayouts::One(Some(a.item_layout())),
			Description::Set(s) => ComposingLayouts::One(Some(s.item_layout())),
			Description::Alias(_, _) => ComposingLayouts::None,
		}
	}
}

pub enum ComposingLayouts<'a, M> {
	Struct(std::slice::Iter<'a, Field<M>>),
	Enum(enumeration::ComposingLayouts<'a, M>),
	One(Option<Ref<Definition<M>>>),
	None,
}

impl<'a, M> Iterator for ComposingLayouts<'a, M> {
	type Item = Ref<Definition<M>>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Struct(fields) => Some(fields.next()?.layout()),
			Self::Enum(layouts) => layouts.next(),
			Self::One(r) => r.take(),
			Self::None => None,
		}
	}
}
