use std::collections::HashMap;

use crate::{
	component,
	node::{self, BindingValueRef},
	prop::{PropertyName, UnknownProperty},
	property_values, vocab, FunctionalPropertyValue, Id, IriIndex, Multiple, PropertyValue,
	PropertyValues, RequiredFunctionalPropertyValue, ResourceType, TId, Type,
};
use derivative::Derivative;
use locspan::Meta;

pub mod array;
pub mod enumeration;
pub mod field;
mod one_or_many;
mod optional;
pub mod primitive;
mod reference;
mod required;
pub mod restriction;
mod set;
mod structure;
pub mod variant;

mod strongly_connected;
mod usages;

pub use array::Array;
pub use enumeration::Enum;
pub use field::Field;
pub use one_or_many::OneOrMany;
pub use optional::Optional;
pub use primitive::{restriction::Restricted as RestrictedPrimitive, Primitive};
pub use reference::Reference;
pub use required::Required;
pub use restriction::{ContainerRestriction, ContainerRestrictions, Restrictions};
pub use set::Set;
pub use structure::Struct;
pub use variant::Variant;

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
	Derived(Primitive),
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
	/// Types represented by this layout.
	ty: PropertyValues<TId<Type>, M>,

	/// Layout description.
	desc: Description<M>,

	/// Intersection.
	intersection_of: FunctionalPropertyValue<Multiple<TId<Layout>, M>, M>,
}

/// Layout description.
#[derive(Debug, Clone)]
pub enum Description<M> {
	/// Never layout.
	Never,

	/// Primitive layout such as a number, a string, etc.
	Primitive(Primitive),

	/// Derived primitive layout.
	Derived(RequiredFunctionalPropertyValue<RestrictedPrimitive<M>, M>),

	/// Reference.
	Reference(RequiredFunctionalPropertyValue<Reference<M>, M>),

	/// Structure.
	Struct(RequiredFunctionalPropertyValue<Struct<M>, M>),

	/// Enumeration.
	Enum(RequiredFunctionalPropertyValue<Enum<M>, M>),

	/// Required.
	Required(RequiredFunctionalPropertyValue<Required<M>, M>),

	/// Option.
	Option(RequiredFunctionalPropertyValue<Optional<M>, M>),

	/// Array.
	Array(RequiredFunctionalPropertyValue<Array<M>, M>),

	/// Set.
	Set(RequiredFunctionalPropertyValue<Set<M>, M>),

	/// One or many.
	OneOrMany(RequiredFunctionalPropertyValue<OneOrMany<M>, M>),

	/// Alias.
	Alias(RequiredFunctionalPropertyValue<TId<Layout>, M>),
}

impl<M> Description<M> {
	pub fn kind(&self) -> Kind {
		match self {
			Self::Never => Kind::Never,
			Self::Primitive(p) => Kind::Primitive(*p),
			Self::Derived(d) => Kind::Derived(d.primitive()),
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

	pub fn collection_item(&self) -> Option<&Meta<TId<Layout>, M>> {
		match self {
			Self::Required(c) => Some(c.item_layout()),
			Self::Option(c) => Some(c.item_layout()),
			Self::Array(c) => Some(c.item_layout()),
			Self::Set(c) => Some(c.item_layout()),
			Self::OneOrMany(c) => Some(c.item_layout()),
			_ => None,
		}
	}

	pub fn is_reference(&self) -> bool {
		matches!(self, Self::Reference(_))
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

	pub fn property_value(&self) -> Option<DescriptionPropertyValue<M>> {
		match self {
			Self::Never | Self::Primitive(_) => None,
			Self::Derived(p) => Some(DescriptionPropertyValue::DerivedFrom(DerivedFrom::new(
				p.sub_properties(),
				p.primitive(),
			))),
			Self::Reference(r) => Some(DescriptionPropertyValue::Reference(r)),
			Self::Struct(s) => Some(DescriptionPropertyValue::Struct(s)),
			Self::Enum(e) => Some(DescriptionPropertyValue::Enum(e)),
			Self::Required(r) => Some(DescriptionPropertyValue::Required(r)),
			Self::Option(o) => Some(DescriptionPropertyValue::Option(o)),
			Self::Array(a) => Some(DescriptionPropertyValue::Array(a)),
			Self::Set(s) => Some(DescriptionPropertyValue::Set(s)),
			Self::OneOrMany(o) => Some(DescriptionPropertyValue::OneOrMany(o)),
			Self::Alias(l) => Some(DescriptionPropertyValue::Alias(l)),
		}
	}

	pub fn restrictions(&self) -> Option<Meta<Restrictions<M>, &M>> {
		match self {
			Self::Derived(d) => d.restrictions().map(|m| m.map(Restrictions::new_primitive)),
			Self::Array(a) => a
				.restrictions()
				.as_ref()
				.map(|m| m.borrow().map(Restrictions::new_container)),
			Self::Set(s) => s
				.restrictions()
				.as_ref()
				.map(|m| m.borrow().map(Restrictions::new_container)),
			Self::OneOrMany(o) => o
				.restrictions()
				.as_ref()
				.map(|m| m.borrow().map(Restrictions::new_container)),
			_ => None,
		}
	}

	pub fn with_restrictions(&self) -> Option<WithRestrictions<M>> {
		self.restrictions().and_then(WithRestrictions::new)
	}
}

/// Values of the `tldr:withRestrictions` property.
pub struct WithRestrictions<'a, M> {
	restrictions: Meta<Restrictions<'a, M>, &'a M>,
}

impl<'a, M> WithRestrictions<'a, M> {
	fn new(restrictions: Meta<Restrictions<'a, M>, &'a M>) -> Option<Self> {
		if restrictions.is_restricted() {
			Some(Self { restrictions })
		} else {
			None
		}
	}

	pub fn iter(&self) -> WithRestrictionsIter<'a, M> {
		WithRestrictionsIter {
			restrictions: Some(self.restrictions),
		}
	}
}

impl<'a, M> IntoIterator for WithRestrictions<'a, M> {
	type IntoIter = WithRestrictionsIter<'a, M>;
	type Item = PropertyValue<Restrictions<'a, M>, &'a M>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

/// Iterator over the values of the `tldr:withRestrictions` property.
pub struct WithRestrictionsIter<'a, M> {
	restrictions: Option<Meta<Restrictions<'a, M>, &'a M>>,
}

impl<'a, M> Iterator for WithRestrictionsIter<'a, M> {
	type Item = PropertyValue<Restrictions<'a, M>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.restrictions
			.take()
			.map(|r| PropertyValue::new(None, r))
	}
}

/// Iterator over the values of the `owl:derivedFrom` property.
pub struct DerivedFrom<'a, M> {
	sub_properties: &'a PropertyValues<(), M>,
	primitive: Primitive,
}

impl<'a, M> DerivedFrom<'a, M> {
	fn new(sub_properties: &'a PropertyValues<(), M>, primitive: Primitive) -> Self {
		Self {
			sub_properties,
			primitive,
		}
	}

	pub fn iter(&self) -> DerivedFromIter<'a, M> {
		DerivedFromIter {
			sub_properties: self.sub_properties.iter(),
			primitive: self.primitive,
		}
	}
}

/// Iterator over the values of the `owl:derivedFrom` property.
pub struct DerivedFromIter<'a, M> {
	sub_properties: property_values::non_functional::Iter<'a, (), M>,
	primitive: Primitive,
}

impl<'a, M> Iterator for DerivedFromIter<'a, M> {
	type Item = PropertyValue<Primitive, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.sub_properties.next().map(|s| {
			PropertyValue::new(
				s.sub_property,
				Meta(self.primitive, s.value.into_metadata()),
			)
		})
	}
}

pub enum DescriptionPropertyValue<'a, M> {
	DerivedFrom(DerivedFrom<'a, M>),
	Reference(&'a RequiredFunctionalPropertyValue<Reference<M>, M>),
	Struct(&'a RequiredFunctionalPropertyValue<Struct<M>, M>),
	Enum(&'a RequiredFunctionalPropertyValue<Enum<M>, M>),
	Required(&'a RequiredFunctionalPropertyValue<Required<M>, M>),
	Option(&'a RequiredFunctionalPropertyValue<Optional<M>, M>),
	Array(&'a RequiredFunctionalPropertyValue<Array<M>, M>),
	Set(&'a RequiredFunctionalPropertyValue<Set<M>, M>),
	OneOrMany(&'a RequiredFunctionalPropertyValue<OneOrMany<M>, M>),
	Alias(&'a RequiredFunctionalPropertyValue<TId<Layout>, M>),
}

impl<'a, M> DescriptionPropertyValue<'a, M> {
	pub fn iter(&self) -> DescriptionPropertyValueIter<'a, M> {
		match self {
			Self::DerivedFrom(i) => DescriptionPropertyValueIter::DerivedFrom(i.iter()),
			Self::Reference(i) => DescriptionPropertyValueIter::Reference(i.iter()),
			Self::Struct(i) => DescriptionPropertyValueIter::Struct(i.iter()),
			Self::Enum(i) => DescriptionPropertyValueIter::Enum(i.iter()),
			Self::Required(i) => DescriptionPropertyValueIter::Required(i.iter()),
			Self::Option(i) => DescriptionPropertyValueIter::Option(i.iter()),
			Self::Array(i) => DescriptionPropertyValueIter::Array(i.iter()),
			Self::Set(i) => DescriptionPropertyValueIter::Set(i.iter()),
			Self::OneOrMany(i) => DescriptionPropertyValueIter::OneOrMany(i.iter()),
			Self::Alias(i) => DescriptionPropertyValueIter::Alias(i.iter()),
		}
	}
}

impl<'a, M> IntoIterator for DescriptionPropertyValue<'a, M> {
	type IntoIter = DescriptionPropertyValueIter<'a, M>;
	type Item = Meta<DescriptionBindingRef<'a, M>, &'a M>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

pub enum DescriptionPropertyValueIter<'a, M> {
	DerivedFrom(DerivedFromIter<'a, M>),
	Reference(property_values::required_functional::Iter<'a, Reference<M>, M>),
	Struct(property_values::required_functional::Iter<'a, Struct<M>, M>),
	Enum(property_values::required_functional::Iter<'a, Enum<M>, M>),
	Required(property_values::required_functional::Iter<'a, Required<M>, M>),
	Option(property_values::required_functional::Iter<'a, Optional<M>, M>),
	Array(property_values::required_functional::Iter<'a, Array<M>, M>),
	Set(property_values::required_functional::Iter<'a, Set<M>, M>),
	OneOrMany(property_values::required_functional::Iter<'a, OneOrMany<M>, M>),
	Alias(property_values::required_functional::Iter<'a, TId<Layout>, M>),
}

impl<'a, M> Iterator for DescriptionPropertyValueIter<'a, M> {
	type Item = Meta<DescriptionBindingRef<'a, M>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::DerivedFrom(v) => v
				.next()
				.map(|s| s.into_class_binding(DescriptionBindingRef::DerivedFrom)),
			Self::Reference(v) => v.next().map(|s| {
				s.into_class_binding(|id, r| DescriptionBindingRef::Reference(id, **r.id_layout()))
			}),
			Self::Struct(v) => v.next().map(|s| {
				s.into_class_binding(|id, s| DescriptionBindingRef::Struct(id, s.fields()))
			}),
			Self::Enum(v) => v.next().map(|s| {
				s.into_class_binding(|id, e| DescriptionBindingRef::Enum(id, e.variants()))
			}),
			Self::Required(v) => v.next().map(|s| {
				s.into_class_binding(|id, c| DescriptionBindingRef::Required(id, **c.item_layout()))
			}),
			Self::Option(v) => v.next().map(|s| {
				s.into_class_binding(|id, c| DescriptionBindingRef::Option(id, **c.item_layout()))
			}),
			Self::Array(v) => v.next().map(|s| {
				s.into_class_binding(|id, c| DescriptionBindingRef::Array(id, **c.item_layout()))
			}),
			Self::Set(v) => v.next().map(|s| {
				s.into_class_binding(|id, c| DescriptionBindingRef::Set(id, **c.item_layout()))
			}),
			Self::OneOrMany(v) => v.next().map(|s| {
				s.into_class_binding(|id, c| {
					DescriptionBindingRef::OneOrMany(id, **c.item_layout())
				})
			}),
			Self::Alias(v) => v
				.next()
				.map(|s| s.into_cloned_class_binding(DescriptionBindingRef::Alias)),
		}
	}
}

impl<M> Definition<M> {
	/// Creates a new layout definition.
	pub fn new(
		ty: PropertyValues<TId<Type>, M>,
		desc: Description<M>,
		intersection_of: FunctionalPropertyValue<Multiple<TId<Layout>, M>, M>,
	) -> Self {
		Self {
			ty,
			desc,
			intersection_of,
		}
	}

	/// Type for which the layout is defined.
	pub fn ty(&self) -> &PropertyValues<TId<Type>, M> {
		&self.ty
	}

	/// Returns the layout description.
	pub fn description(&self) -> &Description<M> {
		&self.desc
	}

	/// Checks if the layout is required.
	///
	/// This means either the `required` container or a non-empty `set`
	/// container.
	pub fn is_required(&self) -> bool {
		self.desc.is_required()
	}

	pub fn composing_layouts<'a>(
		&'a self,
		model: &'a crate::MutableModel<M>,
	) -> ComposingLayouts<'a, M> {
		match self.description() {
			Description::Never | Description::Primitive(_) => ComposingLayouts::None,
			Description::Struct(s) => ComposingLayouts::Fields(model, s.fields().iter()),
			Description::Enum(e) => ComposingLayouts::Enum(e.composing_layouts(model)),
			Description::Reference(r) => ComposingLayouts::One(Some(*r.id_layout().value())),
			Description::Derived(_) => ComposingLayouts::None,
			Description::Option(o) => ComposingLayouts::One(Some(*o.item_layout().value())),
			Description::Required(r) => ComposingLayouts::One(Some(*r.item_layout().value())),
			Description::Array(a) => ComposingLayouts::One(Some(*a.item_layout().value())),
			Description::Set(s) => ComposingLayouts::One(Some(*s.item_layout().value())),
			Description::OneOrMany(s) => ComposingLayouts::One(Some(*s.item_layout().value())),
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
		model: &crate::MutableModel<M>,
	) -> bool {
		match self.description() {
			Description::Reference(_) => true,
			Description::Enum(e) => e.can_be_reference(map, model),
			Description::Option(o) => model.can_be_reference_layout(map, **o.item_layout()),
			Description::Required(r) => model.can_be_reference_layout(map, **r.item_layout()),
			Description::OneOrMany(o) => model.can_be_reference_layout(map, **o.item_layout()),
			Description::Alias(r) => model.can_be_reference_layout(map, **r),
			_ => false,
		}
	}

	pub fn bindings(&self) -> Bindings<M> {
		ClassBindings {
			ty: self.ty.iter(),
			desc: self
				.desc
				.property_value()
				.map(DescriptionPropertyValue::into_iter),
			intersection_of: self.intersection_of.iter(),
			restrictions: self
				.desc
				.with_restrictions()
				.map(WithRestrictions::into_iter),
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
		&'a crate::MutableModel<M>,
		std::slice::Iter<'a, Meta<TId<Field>, M>>,
	),
	Enum(enumeration::ComposingLayouts<'a, M>),
	One(Option<TId<Layout>>),
	None,
}

impl<'a, M> Iterator for ComposingLayouts<'a, M> {
	type Item = TId<Layout>;

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
	For(Option<TId<UnknownProperty>>),
	Description(DescriptionProperty),
	IntersectionOf(Option<TId<UnknownProperty>>),
	WithRestrictions(Option<TId<UnknownProperty>>),
	ArrayListFirst(Option<TId<UnknownProperty>>),
	ArrayListRest(Option<TId<UnknownProperty>>),
	ArrayListNil(Option<TId<UnknownProperty>>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DescriptionProperty {
	DerivedFrom(Option<TId<UnknownProperty>>),
	Fields(Option<TId<UnknownProperty>>),
	Variants(Option<TId<UnknownProperty>>),
	Reference(Option<TId<UnknownProperty>>),
	Required(Option<TId<UnknownProperty>>),
	Option(Option<TId<UnknownProperty>>),
	Set(Option<TId<UnknownProperty>>),
	OneOrMany(Option<TId<UnknownProperty>>),
	Array(Option<TId<UnknownProperty>>),
	Alias(Option<TId<UnknownProperty>>),
}

impl Property {
	pub fn id(&self) -> Id {
		use vocab::{Term, TreeLdr};
		match self {
			Self::For(None) => Id::Iri(IriIndex::Iri(Term::TreeLdr(TreeLdr::LayoutFor))),
			Self::For(Some(p)) => p.id(),
			Self::Description(p) => p.id(),
			Self::IntersectionOf(None) => {
				Id::Iri(IriIndex::Iri(Term::TreeLdr(TreeLdr::IntersectionOf)))
			}
			Self::IntersectionOf(Some(p)) => p.id(),
			Self::WithRestrictions(None) => {
				Id::Iri(IriIndex::Iri(Term::TreeLdr(TreeLdr::WithRestrictions)))
			}
			Self::WithRestrictions(Some(p)) => p.id(),
			Self::ArrayListFirst(None) => {
				Id::Iri(IriIndex::Iri(Term::TreeLdr(TreeLdr::ArrayListFirst)))
			}
			Self::ArrayListFirst(Some(p)) => p.id(),
			Self::ArrayListRest(None) => {
				Id::Iri(IriIndex::Iri(Term::TreeLdr(TreeLdr::ArrayListRest)))
			}
			Self::ArrayListRest(Some(p)) => p.id(),
			Self::ArrayListNil(None) => {
				Id::Iri(IriIndex::Iri(Term::TreeLdr(TreeLdr::ArrayListNil)))
			}
			Self::ArrayListNil(Some(p)) => p.id(),
		}
	}

	pub fn term(&self) -> Option<vocab::Term> {
		use vocab::{Term, TreeLdr};
		match self {
			Self::For(None) => Some(Term::TreeLdr(TreeLdr::LayoutFor)),
			Self::Description(p) => p.term(),
			Self::IntersectionOf(None) => Some(Term::TreeLdr(TreeLdr::IntersectionOf)),
			Self::WithRestrictions(None) => Some(Term::TreeLdr(TreeLdr::WithRestrictions)),
			Self::ArrayListFirst(None) => Some(Term::TreeLdr(TreeLdr::ArrayListFirst)),
			Self::ArrayListRest(None) => Some(Term::TreeLdr(TreeLdr::ArrayListRest)),
			Self::ArrayListNil(None) => Some(Term::TreeLdr(TreeLdr::ArrayListNil)),
			_ => None,
		}
	}

	pub fn name(&self) -> PropertyName {
		match self {
			Self::For(None) => PropertyName::Resource("layout type"),
			Self::For(Some(p)) => PropertyName::Other(*p),
			Self::Description(p) => p.name(),
			Self::IntersectionOf(None) => PropertyName::Resource("intersection"),
			Self::IntersectionOf(Some(p)) => PropertyName::Other(*p),
			Self::WithRestrictions(None) => PropertyName::Resource("layout restrictions"),
			Self::WithRestrictions(Some(p)) => PropertyName::Other(*p),
			Self::ArrayListFirst(None) => {
				PropertyName::Resource("\"array as list\" `first` property")
			}
			Self::ArrayListFirst(Some(p)) => PropertyName::Other(*p),
			Self::ArrayListRest(None) => {
				PropertyName::Resource("\"array as list\" `rest` property")
			}
			Self::ArrayListRest(Some(p)) => PropertyName::Other(*p),
			Self::ArrayListNil(None) => {
				PropertyName::Resource("\"array as list\" empty list value")
			}
			Self::ArrayListNil(Some(p)) => PropertyName::Other(*p),
		}
	}

	pub fn expect_type(&self) -> bool {
		match self {
			Self::For(_) => true,
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
	pub fn id(&self) -> Id {
		use vocab::{Term, TreeLdr};
		match self {
			Self::DerivedFrom(None) => Id::Iri(IriIndex::Iri(Term::TreeLdr(TreeLdr::DerivedFrom))),
			Self::DerivedFrom(Some(p)) => p.id(),
			Self::Reference(None) => Id::Iri(IriIndex::Iri(Term::TreeLdr(TreeLdr::Reference))),
			Self::Reference(Some(p)) => p.id(),
			Self::Fields(None) => Id::Iri(IriIndex::Iri(Term::TreeLdr(TreeLdr::Fields))),
			Self::Fields(Some(p)) => p.id(),
			Self::Variants(None) => Id::Iri(IriIndex::Iri(Term::TreeLdr(TreeLdr::Enumeration))),
			Self::Variants(Some(p)) => p.id(),
			Self::Required(None) => Id::Iri(IriIndex::Iri(Term::TreeLdr(TreeLdr::Required))),
			Self::Required(Some(p)) => p.id(),
			Self::Option(None) => Id::Iri(IriIndex::Iri(Term::TreeLdr(TreeLdr::Option))),
			Self::Option(Some(p)) => p.id(),
			Self::Set(None) => Id::Iri(IriIndex::Iri(Term::TreeLdr(TreeLdr::Set))),
			Self::Set(Some(p)) => p.id(),
			Self::OneOrMany(None) => Id::Iri(IriIndex::Iri(Term::TreeLdr(TreeLdr::OneOrMany))),
			Self::OneOrMany(Some(p)) => p.id(),
			Self::Array(None) => Id::Iri(IriIndex::Iri(Term::TreeLdr(TreeLdr::Array))),
			Self::Array(Some(p)) => p.id(),
			Self::Alias(None) => Id::Iri(IriIndex::Iri(Term::TreeLdr(TreeLdr::Alias))),
			Self::Alias(Some(p)) => p.id(),
		}
	}

	pub fn term(&self) -> Option<vocab::Term> {
		use vocab::{Term, TreeLdr};
		match self {
			Self::DerivedFrom(None) => Some(Term::TreeLdr(TreeLdr::DerivedFrom)),
			Self::Reference(None) => Some(Term::TreeLdr(TreeLdr::Reference)),
			Self::Fields(None) => Some(Term::TreeLdr(TreeLdr::Fields)),
			Self::Variants(None) => Some(Term::TreeLdr(TreeLdr::Enumeration)),
			Self::Required(None) => Some(Term::TreeLdr(TreeLdr::Required)),
			Self::Option(None) => Some(Term::TreeLdr(TreeLdr::Option)),
			Self::Set(None) => Some(Term::TreeLdr(TreeLdr::Set)),
			Self::OneOrMany(None) => Some(Term::TreeLdr(TreeLdr::OneOrMany)),
			Self::Array(None) => Some(Term::TreeLdr(TreeLdr::Array)),
			Self::Alias(None) => Some(Term::TreeLdr(TreeLdr::Alias)),
			_ => None,
		}
	}

	pub fn name(&self) -> PropertyName {
		match self {
			Self::DerivedFrom(None) => PropertyName::Resource("derived primitive layout"),
			Self::DerivedFrom(Some(p)) => PropertyName::Other(*p),
			Self::Reference(None) => PropertyName::Resource("referenced type"),
			Self::Reference(Some(p)) => PropertyName::Other(*p),
			Self::Fields(None) => PropertyName::Resource("structure fields"),
			Self::Fields(Some(p)) => PropertyName::Other(*p),
			Self::Variants(None) => PropertyName::Resource("enum variants"),
			Self::Variants(Some(p)) => PropertyName::Other(*p),
			Self::Required(None) => PropertyName::Resource("required item layout"),
			Self::Required(Some(p)) => PropertyName::Other(*p),
			Self::Option(None) => PropertyName::Resource("optional item layout"),
			Self::Option(Some(p)) => PropertyName::Other(*p),
			Self::Set(None) => PropertyName::Resource("set item layout"),
			Self::Set(Some(p)) => PropertyName::Other(*p),
			Self::OneOrMany(None) => PropertyName::Resource("one or many item layout"),
			Self::OneOrMany(Some(p)) => PropertyName::Other(*p),
			Self::Array(None) => PropertyName::Resource("array item layout"),
			Self::Array(Some(p)) => PropertyName::Other(*p),
			Self::Alias(None) => PropertyName::Resource("alias layout"),
			Self::Alias(Some(p)) => PropertyName::Other(*p),
		}
	}

	pub fn expect_layout(&self) -> bool {
		matches!(
			self,
			Self::DerivedFrom(_)
				| Self::Reference(_)
				| Self::Required(_)
				| Self::Option(_)
				| Self::OneOrMany(_)
				| Self::Array(_) | Self::Alias(_)
		)
	}

	pub fn expect_type(&self) -> bool {
		false
	}
}

#[derive(Debug, Derivative)]
#[derivative(Clone(bound = ""), Copy(bound = ""))]
pub enum DescriptionBindingRef<'a, M> {
	DerivedFrom(Option<TId<UnknownProperty>>, Primitive),
	Reference(Option<TId<UnknownProperty>>, TId<Layout>),
	Struct(Option<TId<UnknownProperty>>, &'a [Meta<TId<Field>, M>]),
	Enum(Option<TId<UnknownProperty>>, &'a [Meta<TId<Variant>, M>]),
	Required(Option<TId<UnknownProperty>>, TId<Layout>),
	Option(Option<TId<UnknownProperty>>, TId<Layout>),
	Array(Option<TId<UnknownProperty>>, TId<Layout>),
	Set(Option<TId<UnknownProperty>>, TId<Layout>),
	OneOrMany(Option<TId<UnknownProperty>>, TId<Layout>),
	Alias(Option<TId<UnknownProperty>>, TId<Layout>),
}

impl<'a, M> DescriptionBindingRef<'a, M> {
	pub fn property(&self) -> DescriptionProperty {
		match self {
			Self::DerivedFrom(p, _) => DescriptionProperty::DerivedFrom(*p),
			Self::Reference(p, _) => DescriptionProperty::Reference(*p),
			Self::Struct(p, _) => DescriptionProperty::Fields(*p),
			Self::Enum(p, _) => DescriptionProperty::Variants(*p),
			Self::Required(p, _) => DescriptionProperty::Required(*p),
			Self::Option(p, _) => DescriptionProperty::Option(*p),
			Self::Array(p, _) => DescriptionProperty::Array(*p),
			Self::Set(p, _) => DescriptionProperty::Set(*p),
			Self::OneOrMany(p, _) => DescriptionProperty::OneOrMany(*p),
			Self::Alias(p, _) => DescriptionProperty::Alias(*p),
		}
	}

	pub fn value(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::DerivedFrom(_, p) => BindingValueRef::Type(p.ty()),
			Self::Reference(_, v) => BindingValueRef::Layout(*v),
			Self::Struct(_, v) => BindingValueRef::Fields(v),
			Self::Enum(_, v) => BindingValueRef::Variants(v),
			Self::Required(_, v) => BindingValueRef::Layout(*v),
			Self::Option(_, v) => BindingValueRef::Layout(*v),
			Self::Array(_, v) => BindingValueRef::Layout(*v),
			Self::Set(_, v) => BindingValueRef::Layout(*v),
			Self::OneOrMany(_, v) => BindingValueRef::Layout(*v),
			Self::Alias(_, v) => BindingValueRef::Layout(*v),
		}
	}
}

pub enum ClassBindingRef<'a, M> {
	For(Option<TId<UnknownProperty>>, TId<crate::Type>),
	Description(DescriptionBindingRef<'a, M>),
	IntersectionOf(Option<TId<UnknownProperty>>, &'a Multiple<TId<Layout>, M>),
	WithRestrictions(Option<TId<UnknownProperty>>, Restrictions<'a, M>),
	ArraySemantics(array::Binding),
}

pub type BindingRef<'a, M> = ClassBindingRef<'a, M>;

impl<'a, M> ClassBindingRef<'a, M> {
	pub fn property(&self) -> Property {
		match self {
			Self::For(p, _) => Property::For(*p),
			Self::Description(d) => Property::Description(d.property()),
			Self::IntersectionOf(p, _) => Property::IntersectionOf(*p),
			Self::WithRestrictions(p, _) => Property::WithRestrictions(*p),
			Self::ArraySemantics(b) => b.property(),
		}
	}

	pub fn value(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::For(_, v) => BindingValueRef::Type(*v),
			Self::Description(d) => d.value(),
			Self::IntersectionOf(_, v) => {
				BindingValueRef::LayoutList(node::MultipleIdValueRef::Multiple(*v))
			}
			Self::WithRestrictions(_, v) => BindingValueRef::LayoutRestrictions(*v),
			Self::ArraySemantics(b) => b.value(),
		}
	}
}

pub struct ClassBindings<'a, M> {
	ty: property_values::non_functional::Iter<'a, TId<crate::Type>, M>,
	desc: Option<DescriptionPropertyValueIter<'a, M>>,
	intersection_of: property_values::functional::Iter<'a, Multiple<TId<crate::Layout>, M>, M>,
	restrictions: Option<WithRestrictionsIter<'a, M>>,
	array_semantics: array::Bindings<'a, M>,
}

pub type Bindings<'a, M> = ClassBindings<'a, M>;

impl<'a, M> Iterator for ClassBindings<'a, M> {
	type Item = Meta<ClassBindingRef<'a, M>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.ty
			.next()
			.map(|v| v.into_cloned_class_binding(ClassBindingRef::For))
			.or_else(|| {
				self.desc
					.as_mut()
					.and_then(DescriptionPropertyValueIter::next)
					.map(|d| d.map(ClassBindingRef::Description))
					.or_else(|| {
						self.intersection_of
							.next()
							.map(|v| v.into_class_binding(ClassBindingRef::IntersectionOf))
							.or_else(|| {
								self.restrictions
									.as_mut()
									.and_then(|v| {
										v.next().map(|v| {
											v.into_class_binding(ClassBindingRef::WithRestrictions)
										})
									})
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
