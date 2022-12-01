use std::collections::HashMap;

use crate::{
	component,
	node::{self, BindingValueRef},
	vocab, MetaOption, Multiple, ResourceType, TId, Type,
};
use derivative::Derivative;
use locspan::Meta;

pub mod array;
pub mod enumeration;
pub mod value_enumeration;
mod one_or_many;
mod optional;
pub mod primitive;
mod reference;
mod required;
pub mod restriction;
mod set;
pub mod structure;

mod strongly_connected;
mod usages;

pub use array::Array;
pub use enumeration::Enum;
pub use value_enumeration::ValueEnum;
pub use one_or_many::OneOrMany;
pub use optional::Optional;
pub use primitive::{restriction::Restricted as RestrictedPrimitive, Primitive};
pub use reference::Reference;
pub use required::Required;
pub use restriction::{ContainerRestriction, ContainerRestrictions, Restrictions};
pub use set::Set;
pub use structure::Struct;

pub use strongly_connected::StronglyConnectedLayouts;
pub use usages::Usages;

pub struct Layout;

impl ResourceType for Layout {
	const TYPE: crate::Type =
		crate::Type::Resource(Some(node::Type::Component(Some(component::Type::Layout))));

	fn check<M>(resource: &crate::node::Definition<M>) -> bool {
		resource.is_layout()
	}
}

impl<'a, M> crate::Ref<'a, Layout, M> {
	pub fn as_component(&self) -> &'a Meta<component::Definition<M>, M> {
		self.as_resource().as_component().unwrap()
	}

	pub fn as_layout(&self) -> &'a Meta<Definition<M>, M> {
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
pub struct Definition<M> {
	/// Type represented by this layout.
	ty: MetaOption<TId<Type>, M>,

	/// Layout description.
	desc: Meta<Description<M>, M>,

	/// Intersection.
	intersection_of: MetaOption<Multiple<TId<Layout>, M>, M>,
}

/// Layout description.
#[derive(Debug, Clone)]
pub enum Description<M> {
	/// Never layout.
	Never,

	/// Primitive layout, such as a number, a string, etc.
	Primitive(RestrictedPrimitive<M>),

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
	Alias(TId<Layout>),
}

impl<M> Description<M> {
	pub fn kind(&self) -> Kind {
		match self {
			Self::Never => Kind::Never,
			Self::Primitive(n) => Kind::Primitive(n.primitive()),
			Self::Reference(_) => Kind::Reference,
			Self::Struct(_) => Kind::Struct,
			Self::Enum(_) => Kind::Enum,
			Self::Required(_) => Kind::Required,
			Self::Option(_) => Kind::Option,
			Self::Array(_) => Kind::Array,
			Self::Set(_) => Kind::Set,
			Self::OneOrMany(_) => Kind::OneOrMany,
			Self::Alias(_) => Kind::Alias,
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

	pub fn array_semantics(&self) -> Option<&array::Semantics<M>> {
		match self {
			Self::Array(a) => a.semantics(),
			_ => None,
		}
	}

	pub fn as_binding_ref(&self) -> Option<DescriptionBindingRef<M>> {
		match self {
			Self::Never => None,
			Self::Primitive(p) => p.as_binding_ref(),
			Self::Reference(r) => Some(DescriptionBindingRef::Reference(**r.id_layout())),
			Self::Struct(s) => Some(DescriptionBindingRef::Struct(s.fields())),
			Self::Enum(e) => Some(DescriptionBindingRef::Enum(e.variants())),
			Self::Required(r) => Some(DescriptionBindingRef::Required(**r.item_layout())),
			Self::Option(o) => Some(DescriptionBindingRef::Option(**o.item_layout())),
			Self::Array(a) => Some(DescriptionBindingRef::Array(**a.item_layout())),
			Self::Set(s) => Some(DescriptionBindingRef::Set(**s.item_layout())),
			Self::OneOrMany(o) => Some(DescriptionBindingRef::OneOrMany(**o.item_layout())),
			Self::Alias(l) => Some(DescriptionBindingRef::Alias(*l)),
		}
	}

	pub fn restrictions(&self) -> Option<Restrictions<M>> {
		match self {
			Self::Primitive(p) => p.restrictions().map(Restrictions::new_primitive),
			Self::Array(a) => a
				.restrictions()
				.as_restricted()
				.map(Restrictions::new_container),
			Self::Set(s) => s
				.restrictions()
				.as_restricted()
				.map(Restrictions::new_container),
			Self::OneOrMany(o) => o
				.restrictions()
				.as_restricted()
				.map(Restrictions::new_container),
			_ => None,
		}
	}
}

impl<M> Definition<M> {
	/// Creates a new layout definition.
	pub fn new(
		ty: MetaOption<TId<Type>, M>,
		desc: Meta<Description<M>, M>,
		intersection_of: MetaOption<Multiple<TId<Layout>, M>, M>,
	) -> Self {
		Self {
			ty,
			desc,
			intersection_of,
		}
	}

	/// Type for which the layout is defined.
	pub fn ty(&self) -> Option<TId<Type>> {
		self.ty.value().cloned()
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

	pub fn composing_layouts<'a>(&'a self, model: &'a crate::Model<M>) -> ComposingLayouts<'a, M> {
		match self.description().value() {
			Description::Never => ComposingLayouts::None,
			Description::Struct(s) => ComposingLayouts::Fields(model, s.fields().iter()),
			Description::Enum(e) => ComposingLayouts::Enum(e.composing_layouts(model)),
			Description::Reference(r) => ComposingLayouts::One(Some(r.id_layout())),
			Description::Primitive(_) => ComposingLayouts::None,
			Description::Option(o) => ComposingLayouts::One(Some(o.item_layout())),
			Description::Required(r) => ComposingLayouts::One(Some(r.item_layout())),
			Description::Array(a) => ComposingLayouts::One(Some(a.item_layout())),
			Description::Set(s) => ComposingLayouts::One(Some(s.item_layout())),
			Description::OneOrMany(s) => ComposingLayouts::One(Some(s.item_layout())),
			Description::Alias(_) => ComposingLayouts::None,
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
			Description::Option(o) => model.can_be_reference_layout(map, **o.item_layout()),
			Description::Required(r) => model.can_be_reference_layout(map, **r.item_layout()),
			Description::OneOrMany(o) => model.can_be_reference_layout(map, **o.item_layout()),
			Description::Alias(r) => model.can_be_reference_layout(map, *r),
			_ => false,
		}
	}

	pub fn bindings(&self) -> Bindings<M> {
		ClassBindings {
			ty: self.ty.as_ref(),
			desc: self
				.desc
				.as_binding_ref()
				.map(|b| Meta(b, self.desc.metadata())),
			intersection_of: self.intersection_of.as_ref(),
			restrictions: self
				.desc
				.restrictions()
				.map(|r| Meta(r, self.desc.metadata())),
			array_semantics: self
				.desc
				.array_semantics()
				.map(|s| s.bindings())
				.unwrap_or_default(),
		}
	}
}

pub enum ComposingLayouts<'a, M> {
	Fields(
		&'a crate::Model<M>,
		std::slice::Iter<'a, Meta<TId<structure::Field>, M>>,
	),
	Enum(enumeration::ComposingLayouts<'a, M>),
	One(Option<&'a Meta<TId<Layout>, M>>),
	None,
}

impl<'a, M> Iterator for ComposingLayouts<'a, M> {
	type Item = &'a Meta<TId<Layout>, M>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Fields(model, fields) => fields.next().map(|f| model.get(**f).unwrap().format()),
			Self::Enum(e) => e.next(),
			Self::One(r) => r.take(),
			Self::None => None,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Property {
	For,
	Description(DescriptionProperty),
	IntersectionOf,
	WithRestrictions,
	ArrayListFirst,
	ArrayListRest,
	ArrayListNil,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DescriptionProperty {
	DerivedFrom,
	Fields,
	Variants,
	Reference,
	Required,
	Option,
	Set,
	OneOrMany,
	Array,
	Alias,
}

impl Property {
	pub fn term(&self) -> vocab::Term {
		use vocab::{Term, TreeLdr};
		match self {
			Self::For => Term::TreeLdr(TreeLdr::LayoutFor),
			Self::Description(p) => p.term(),
			Self::IntersectionOf => Term::TreeLdr(TreeLdr::IntersectionOf),
			Self::WithRestrictions => Term::TreeLdr(TreeLdr::WithRestrictions),
			Self::ArrayListFirst => Term::TreeLdr(TreeLdr::ArrayListFirst),
			Self::ArrayListRest => Term::TreeLdr(TreeLdr::ArrayListRest),
			Self::ArrayListNil => Term::TreeLdr(TreeLdr::ArrayListNil),
		}
	}

	pub fn name(&self) -> &'static str {
		match self {
			Self::For => "layout type",
			Self::Description(p) => p.name(),
			Self::IntersectionOf => "intersection",
			Self::WithRestrictions => "layout restrictions",
			Self::ArrayListFirst => "\"array as list\" `first` property",
			Self::ArrayListRest => "\"array as list\" `rest` property",
			Self::ArrayListNil => "\"array as list\" empty list value",
		}
	}

	pub fn expect_type(&self) -> bool {
		match self {
			Self::For => true,
			Self::Description(p) => p.expect_type(),
			_ => false,
		}
	}

	pub fn expect_layout(&self) -> bool {
		match self {
			Self::Description(p) => p.expect_layout(),
			_ => false,
		}
	}
}

impl DescriptionProperty {
	pub fn term(&self) -> vocab::Term {
		use vocab::{Term, TreeLdr};
		match self {
			Self::DerivedFrom => Term::TreeLdr(TreeLdr::DerivedFrom),
			Self::Reference => Term::TreeLdr(TreeLdr::Reference),
			Self::Fields => Term::TreeLdr(TreeLdr::Fields),
			Self::Variants => Term::TreeLdr(TreeLdr::Enumeration),
			Self::Required => Term::TreeLdr(TreeLdr::Required),
			Self::Option => Term::TreeLdr(TreeLdr::Option),
			Self::Set => Term::TreeLdr(TreeLdr::Set),
			Self::OneOrMany => Term::TreeLdr(TreeLdr::OneOrMany),
			Self::Array => Term::TreeLdr(TreeLdr::Array),
			Self::Alias => Term::TreeLdr(TreeLdr::Alias),
		}
	}

	pub fn name(&self) -> &'static str {
		match self {
			Self::DerivedFrom => "derived primitive layout",
			Self::Reference => "referenced type",
			Self::Fields => "structure fields",
			Self::Variants => "enum variants",
			Self::Required => "required item layout",
			Self::Option => "optional item layout",
			Self::Set => "set item layout",
			Self::OneOrMany => "one or many item layout",
			Self::Array => "array item layout",
			Self::Alias => "alias layout",
		}
	}

	pub fn expect_layout(&self) -> bool {
		matches!(
			self,
			Self::DerivedFrom
				| Self::Reference
				| Self::Required | Self::Option
				| Self::OneOrMany
				| Self::Array | Self::Alias
		)
	}

	pub fn expect_type(&self) -> bool {
		false
	}
}

#[derive(Debug, Derivative)]
#[derivative(Clone(bound = ""), Copy(bound = ""))]
pub enum DescriptionBindingRef<'a, M> {
	DerivedFrom(Primitive),
	Reference(TId<Layout>),
	Struct(&'a [Meta<TId<structure::Field>, M>]),
	Enum(&'a [Meta<TId<enumeration::Variant>, M>]),
	Required(TId<Layout>),
	Option(TId<Layout>),
	Array(TId<Layout>),
	Set(TId<Layout>),
	OneOrMany(TId<Layout>),
	Alias(TId<Layout>),
}

impl<'a, M> DescriptionBindingRef<'a, M> {
	pub fn property(&self) -> DescriptionProperty {
		match self {
			Self::DerivedFrom(_) => DescriptionProperty::DerivedFrom,
			Self::Reference(_) => DescriptionProperty::Reference,
			Self::Struct(_) => DescriptionProperty::Fields,
			Self::Enum(_) => DescriptionProperty::Variants,
			Self::Required(_) => DescriptionProperty::Required,
			Self::Option(_) => DescriptionProperty::Option,
			Self::Array(_) => DescriptionProperty::Array,
			Self::Set(_) => DescriptionProperty::Set,
			Self::OneOrMany(_) => DescriptionProperty::OneOrMany,
			Self::Alias(_) => DescriptionProperty::Alias,
		}
	}

	pub fn value(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::DerivedFrom(p) => BindingValueRef::Type(p.ty()),
			Self::Reference(v) => BindingValueRef::Layout(*v),
			Self::Struct(v) => BindingValueRef::Fields(v),
			Self::Enum(v) => BindingValueRef::Variants(v),
			Self::Required(v) => BindingValueRef::Layout(*v),
			Self::Option(v) => BindingValueRef::Layout(*v),
			Self::Array(v) => BindingValueRef::Layout(*v),
			Self::Set(v) => BindingValueRef::Layout(*v),
			Self::OneOrMany(v) => BindingValueRef::Layout(*v),
			Self::Alias(v) => BindingValueRef::Layout(*v),
		}
	}
}

pub enum ClassBindingRef<'a, M> {
	For(TId<crate::Type>),
	Description(DescriptionBindingRef<'a, M>),
	IntersectionOf(&'a Multiple<TId<Layout>, M>),
	WithRestrictions(Restrictions<'a, M>),
	ArraySemantics(array::Binding),
}

pub type BindingRef<'a, M> = ClassBindingRef<'a, M>;

impl<'a, M> ClassBindingRef<'a, M> {
	pub fn property(&self) -> Property {
		match self {
			Self::For(_) => Property::For,
			Self::Description(d) => Property::Description(d.property()),
			Self::IntersectionOf(_) => Property::IntersectionOf,
			Self::WithRestrictions(_) => Property::WithRestrictions,
			Self::ArraySemantics(b) => b.property(),
		}
	}

	pub fn value(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::For(v) => BindingValueRef::Type(*v),
			Self::Description(d) => d.value(),
			Self::IntersectionOf(v) => BindingValueRef::Layouts(v),
			Self::WithRestrictions(v) => BindingValueRef::LayoutRestrictions(*v),
			Self::ArraySemantics(b) => b.value(),
		}
	}
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct ClassBindings<'a, M> {
	ty: Option<&'a Meta<TId<crate::Type>, M>>,
	desc: Option<Meta<DescriptionBindingRef<'a, M>, &'a M>>,
	intersection_of: Option<&'a Meta<Multiple<TId<Layout>, M>, M>>,
	restrictions: Option<Meta<Restrictions<'a, M>, &'a M>>,
	array_semantics: array::Bindings<'a, M>,
}

pub type Bindings<'a, M> = ClassBindings<'a, M>;

impl<'a, M> Iterator for ClassBindings<'a, M> {
	type Item = Meta<ClassBindingRef<'a, M>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.ty
			.take()
			.map(|v| v.borrow().into_cloned_value().map(ClassBindingRef::For))
			.or_else(|| {
				self.desc
					.take()
					.map(|v| v.map(ClassBindingRef::Description))
					.or_else(|| {
						self.intersection_of
							.take()
							.map(|v| v.borrow().map(ClassBindingRef::IntersectionOf))
							.or_else(|| {
								self.restrictions
									.take()
									.map(|v| v.map(ClassBindingRef::WithRestrictions))
									.or_else(|| {
										self.array_semantics
											.next()
											.map(|v| v.map(ClassBindingRef::ArraySemantics))
									})
							})
					})
			})
	}
}
