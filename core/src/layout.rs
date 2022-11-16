use std::collections::HashMap;

use crate::{
	BlankIdIndex, Id, IriIndex, MetaOption, Name, TId, ResourceType, component, Type, vocab
};
use locspan::Meta;
use rdf_types::Subject;

pub mod restriction;
pub mod array;
pub mod enumeration;
mod one_or_many;
mod optional;
pub mod primitive;
mod reference;
mod required;
mod set;
pub mod field;
mod structure;

mod strongly_connected;
mod usages;

pub use restriction::{Restriction, Restrictions};
pub use array::Array;
pub use enumeration::{Enum, Variant};
pub use one_or_many::OneOrMany;
pub use optional::Optional;
pub use primitive::{restriction::Restricted as RestrictedPrimitive, Primitive};
pub use reference::Reference;
pub use required::Required;
pub use set::Set;
pub use structure::{Field, Struct};

pub use strongly_connected::StronglyConnectedLayouts;
pub use usages::Usages;

pub struct Layout;

impl ResourceType for Layout {
	const TYPE: crate::Type = crate::Type::Component(component::Type::Layout);

	fn check<M>(resource: &crate::node::Definition<M>) -> bool {
		resource.is_layout()
	}
}

impl<'a, M> crate::Ref<'a, Layout, M> {
	pub fn as_layout(&self) -> &'a Definition<M> {
		self.as_resource().as_layout().unwrap()
	}
}

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
	OneOrMany,
	Alias,
}

/// Layout definition.
#[derive(Debug)]
pub struct Definition<M, I = IriIndex, B = BlankIdIndex> {
	/// Identifier of the layout.
	id: Subject<I, B>,

	/// Type represented by this layout.
	ty: MetaOption<TId<Type>, M>,

	/// Layout description.
	desc: Meta<Description<M>, M>
}

/// Layout description.
#[derive(Debug, Clone)]
pub enum Description<M> {
	/// Never layout.
	Never(MetaOption<Name, M>),

	/// Primitive layout, such as a number, a string, etc.
	Primitive(RestrictedPrimitive<M>, MetaOption<Name, M>),

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

	/// One or many.
	OneOrMany(OneOrMany<M>),

	/// Alias.
	Alias(Meta<Name, M>, TId<Layout>),
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
			Self::OneOrMany(_) => Kind::OneOrMany,
			Self::Alias(_, _) => Kind::Alias,
		}
	}

	/// Checks if this layout is required.
	///
	/// This means either the `required` container or a non-empty `set`/`array`
	/// container.
	pub fn is_required(&self) -> bool {
		match self {
			Self::Required(_) => true,
			Self::Set(s) => s.is_required(),
			Self::OneOrMany(s) => s.is_required(),
			Self::Array(a) => a.is_required(),
			_ => false,
		}
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
			Self::OneOrMany(s) => s.set_name(new_name, metadata),
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
			Description::OneOrMany(s) => s.into_name(),
			Description::Alias(n, _) => n.into(),
		}
	}
}

impl<M> Definition<M> {
	/// Creates a new layout definition.
	pub fn new(
		id: Id,
		ty: MetaOption<TId<Type>, M>,
		desc: Meta<Description<M>, M>
	) -> Self {
		Self {
			id,
			ty,
			desc
		}
	}

	/// Type for which the layout is defined.
	pub fn ty(&self) -> Option<TId<Type>> {
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
			Description::OneOrMany(s) => s.name(),
			Description::Alias(n, _) => Some(n),
		}
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
			Description::OneOrMany(s) => ComposingLayouts::One(Some(s.item_layout())),
			Description::Alias(_, _) => ComposingLayouts::None,
		}
	}

	/// Checks if this layout is either:
	///   - a reference,
	///   - an enum with a reference variant,
	///   - an option layout with a reference item,
	///   - a required layout with a reference item,
	///   - a OneOrMany layout with a reference item,
	///   - an alias to a layout satisfying one of these conditions.
	///
	/// The map stores the result of this method for other layouts and is
	/// updated to avoid loops.
	pub fn can_be_reference(
		&self,
		map: &mut HashMap<TId<Layout>, bool>,
		model: &crate::Model<M>,
	) -> bool {
		match self.description().value() {
			Description::Reference(_) => true,
			Description::Enum(e) => e.can_be_reference(map, model),
			Description::Option(o) => model.can_be_reference_layout(map, o.item_layout()),
			Description::Required(r) => model.can_be_reference_layout(map, r.item_layout()),
			Description::OneOrMany(o) => model.can_be_reference_layout(map, o.item_layout()),
			Description::Alias(_, r) => model.can_be_reference_layout(map, *r),
			_ => false,
		}
	}
}

pub enum ComposingLayouts<'a, M> {
	Struct(std::slice::Iter<'a, Field<M>>),
	Enum(enumeration::ComposingLayouts<'a, M>),
	One(Option<TId<Layout>>),
	None,
}

impl<'a, M> Iterator for ComposingLayouts<'a, M> {
	type Item = TId<Layout>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Struct(fields) => Some(fields.next()?.layout()),
			Self::Enum(layouts) => layouts.next(),
			Self::One(r) => r.take(),
			Self::None => None,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Property {
	For,
	Reference,
	Fields,
	Variants,
	Required,
	Option,
	Set,
	OneOrMany,
	Array,
	Alias,
	WithRestrictions,
	ArrayListFirst,
	ArrayListRest,
	ArrayListNil,
}

impl Property {
	pub fn term(&self) -> vocab::Term {
		use vocab::{Term, TreeLdr};
		match self {
			Self::For => Term::TreeLdr(TreeLdr::LayoutFor),
			Self::Reference => Term::TreeLdr(TreeLdr::Reference),
			Self::Fields => Term::TreeLdr(TreeLdr::Fields),
			Self::Variants => Term::TreeLdr(TreeLdr::Enumeration),
			Self::Required => Term::TreeLdr(TreeLdr::Required),
			Self::Option => Term::TreeLdr(TreeLdr::Option),
			Self::Set => Term::TreeLdr(TreeLdr::Set),
			Self::OneOrMany => Term::TreeLdr(TreeLdr::OneOrMany),
			Self::Array => Term::TreeLdr(TreeLdr::Array),
			Self::Alias => Term::TreeLdr(TreeLdr::Alias),
			Self::WithRestrictions => Term::TreeLdr(TreeLdr::WithRestrictions),
			Self::ArrayListFirst => Term::TreeLdr(TreeLdr::ArrayListFirst),
			Self::ArrayListRest => Term::TreeLdr(TreeLdr::ArrayListRest),
			Self::ArrayListNil => Term::TreeLdr(TreeLdr::ArrayListNil),
		}
	}

	pub fn name(&self) -> &'static str {
		match self {
			Self::For => "layout type",
			Self::Reference => "referenced type",
			Self::Fields => "structure fields",
			Self::Variants => "enum variants",
			Self::Required => "required item layout",
			Self::Option => "optional item layout",
			Self::Set => "set item layout",
			Self::OneOrMany => "one or many item layout",
			Self::Array => "array item layout",
			Self::Alias => "alias layout",
			Self::WithRestrictions => "layout restrictions",
			Self::ArrayListFirst => "\"array as list\" `first` property",
			Self::ArrayListRest => "\"array as list\" `rest` property",
			Self::ArrayListNil => "\"array as list\" empty list value",
		}
	}
}