use std::cmp::Ordering;

use locspan::Meta;
use treeldr::{metadata::Merge, ty::data::RegExp, value, vocab::Object, Id, MetaOption, Name};

use crate::{
	component,
	context::{HasType, MapIds},
	error::NodeTypeInvalid,
	layout, list, prop, rdf, ty, property_values, Error, ObjectAsId,
	PropertyValues
};
pub use treeldr::node::{Property, Type};

#[derive(Debug, Clone)]
pub struct Data<M> {
	pub id: Id,
	pub metadata: M,
	pub type_: PropertyValues<crate::Type, M>,
	pub label: PropertyValues<String, M>,
	pub comment: PropertyValues<String, M>,
}

impl<M> Data<M> {
	pub fn new(id: Id, metadata: M) -> Self {
		Self {
			id,
			metadata,
			type_: PropertyValues::default(),
			label: PropertyValues::default(),
			comment: PropertyValues::default(),
		}
	}

	pub fn bindings(&self) -> ClassBindings<M> {
		ClassBindings {
			type_: self.type_.iter(),
			label: self.label.iter(),
			comment: self.comment.iter(),
		}
	}
}

impl<M> MapIds for Data<M> {
	fn map_ids(&mut self, f: impl Fn(Id, Option<crate::Property>) -> Id) {
		self.id = f(self.id, Some(Property::Self_.into()))
	}
}

#[derive(Clone)]
pub struct Definition<M> {
	data: Data<M>,
	ty: ty::Definition<M>,
	datatype_restriction: ty::datatype::restriction::Definition<M>,
	property: prop::Definition<M>,
	component: component::Definition<M>,
	layout_restriction: layout::restriction::Definition<M>,
	list: list::Definition<M>,
}

impl<M> Definition<M> {
	pub fn new(id: Id, metadata: M) -> Self {
		Self {
			data: Data::new(id, metadata),
			ty: ty::Definition::new(),
			datatype_restriction: ty::datatype::restriction::Definition::new(),
			property: prop::Definition::new(),
			component: component::Definition::new(),
			layout_restriction: layout::restriction::Definition::new(),
			list: list::Definition::new(),
		}
	}

	pub fn id(&self) -> Id {
		self.data.id
	}

	pub fn metadata(&self) -> &M {
		&self.data.metadata
	}

	pub fn metadata_mut(&mut self) -> &mut M {
		&mut self.data.metadata
	}

	pub fn type_(&self) -> &PropertyValues<crate::Type, M> {
		&self.data.type_
	}

	pub fn type_mut(&mut self) -> &mut PropertyValues<crate::Type, M> {
		&mut self.data.type_
	}

	pub fn has_type(&self, context: &crate::Context<M>, type_: impl Into<crate::Type>) -> bool {
		self.data.has_type(context, type_)
	}

	pub fn label(&self) -> &PropertyValues<String, M> {
		&self.data.label
	}

	pub fn label_mut(&mut self) -> &mut PropertyValues<String, M> {
		&mut self.data.label
	}

	pub fn comment(&self) -> &PropertyValues<String, M> {
		&self.data.comment
	}

	pub fn comment_mut(&mut self) -> &mut PropertyValues<String, M> {
		&mut self.data.comment
	}

	pub fn as_resource(&self) -> &Data<M> {
		&self.data
	}

	pub fn as_type(&self) -> &ty::Definition<M> {
		&self.ty
	}

	pub fn as_datatype(&self) -> &ty::datatype::Definition<M> {
		self.ty.as_datatype()
	}

	pub fn as_restriction(&self) -> &ty::restriction::Definition<M> {
		self.ty.as_restriction()
	}

	pub fn as_datatype_restriction(&self) -> &ty::datatype::restriction::Definition<M> {
		&self.datatype_restriction
	}

	pub fn as_property(&self) -> &prop::Definition<M> {
		&self.property
	}

	pub fn as_component(&self) -> &component::Definition<M> {
		&self.component
	}

	pub fn as_formatted(&self) -> &component::formatted::Definition<M> {
		self.component.as_formatted()
	}

	pub fn as_layout(&self) -> &layout::Definition<M> {
		self.component.as_layout()
	}

	pub fn as_layout_field(&self) -> &layout::field::Definition<M> {
		self.component.as_layout_field()
	}

	pub fn as_layout_variant(&self) -> &layout::variant::Definition {
		self.component.as_layout_variant()
	}

	pub fn as_layout_restriction(&self) -> &layout::restriction::Definition<M> {
		&self.layout_restriction
	}

	pub fn as_list(&self) -> &list::Definition<M> {
		&self.list
	}

	pub fn as_type_mut(&mut self) -> &mut ty::Definition<M> {
		&mut self.ty
	}

	pub fn as_datatype_mut(&mut self) -> &mut ty::datatype::Definition<M> {
		self.ty.as_datatype_mut()
	}

	pub fn as_restriction_mut(&mut self) -> &mut ty::restriction::Definition<M> {
		self.ty.as_restriction_mut()
	}

	pub fn as_datatype_restriction_mut(&mut self) -> &mut ty::datatype::restriction::Definition<M> {
		&mut self.datatype_restriction
	}

	pub fn as_property_mut(&mut self) -> &mut prop::Definition<M> {
		&mut self.property
	}

	pub fn as_component_mut(&mut self) -> &mut component::Definition<M> {
		&mut self.component
	}

	pub fn as_formatted_mut(&mut self) -> &mut component::formatted::Definition<M> {
		self.component.as_formatted_mut()
	}

	pub fn as_layout_mut(&mut self) -> &mut layout::Definition<M> {
		self.component.as_layout_mut()
	}

	pub fn as_layout_field_mut(&mut self) -> &mut layout::field::Definition<M> {
		self.component.as_layout_field_mut()
	}

	pub fn as_layout_variant_mut(&mut self) -> &mut layout::variant::Definition {
		self.component.as_layout_variant_mut()
	}

	pub fn as_layout_restriction_mut(&mut self) -> &mut layout::restriction::Definition<M> {
		&mut self.layout_restriction
	}

	pub fn as_list_mut(&mut self) -> &mut list::Definition<M> {
		&mut self.list
	}

	pub fn bindings(&self) -> Bindings<M> {
		Bindings {
			data: self.data.bindings(),
			class: self.ty.bindings(),
			datatype_restriction: self.datatype_restriction.bindings(),
			property: self.property.bindings(),
			component: self.component.bindings(),
			layout_restriction: self.layout_restriction.bindings(),
			list: self.list.bindings(),
		}
	}

	pub fn set(
		&mut self,
		prop_cmp: impl Fn(Id, Id) -> Option<Ordering>,
		prop: impl Into<crate::Property>,
		value: Meta<Object<M>, M>,
	) -> Result<(), Error<M>>
	where
		M: Merge,
	{
		match prop.into() {
			crate::Property::Resource(prop) => match prop {
				Property::Self_ => (),
				Property::Type => self.type_mut().insert(None, prop_cmp, rdf::from::expect_type(value)?),
				Property::Label => self.label_mut().insert(None, prop_cmp, rdf::from::expect_string(value)?),
				Property::Comment => self.comment_mut().insert(None, prop_cmp, rdf::from::expect_string(value)?),
				Property::Class(prop) => self.as_type_mut().set(prop_cmp, prop, value)?,
				Property::DatatypeRestriction(prop) => {
					self.as_datatype_restriction_mut().set(prop_cmp, prop, value)?
				}
				Property::Property(prop) => self.as_property_mut().set(prop_cmp, prop, value)?,
				Property::Component(prop) => self.as_component_mut().set(prop_cmp, prop, value)?,
				Property::LayoutRestriction(prop) => {
					self.as_layout_restriction_mut().set(prop_cmp, prop, value)?
				}
				Property::List(prop) => self.as_list_mut().set(prop_cmp, prop, value)?,
			},
			crate::Property::Other(_) => {
				// Ignore unknown property.
				// TODO store them somewhere.
			}
		}

		Ok(())
	}
}

impl<M: Merge> MapIds for Definition<M> {
	fn map_ids(&mut self, f: impl Fn(Id, Option<crate::Property>) -> Id) {
		self.data.map_ids(&f);
		self.ty.map_ids(&f);
		self.datatype_restriction.map_ids(&f);
		self.property.map_ids(&f);
		self.component.map_ids(&f);
		self.layout_restriction.map_ids(&f);
		self.list.map_ids(f)
	}
}

impl<M: Clone> Definition<M> {
	pub fn require_type(
		&self,
		context: &crate::Context<M>,
	) -> Result<&ty::Definition<M>, NodeTypeInvalid<M>> {
		if self.has_type(context, Type::Class(None)) {
			Ok(self.as_type())
		} else {
			Err(NodeTypeInvalid {
				id: self.data.id,
				expected: Type::Class(None).into(),
				found: self.data.type_.clone(),
			})
		}
	}

	pub fn require_type_id(
		&self,
		context: &crate::Context<M>,
	) -> Result<treeldr::TId<crate::Type>, NodeTypeInvalid<M>> {
		if self.has_type(context, Type::Class(None)) {
			Ok(treeldr::TId::new(self.data.id))
		} else {
			Err(NodeTypeInvalid {
				id: self.data.id,
				expected: Type::Class(None).into(),
				found: self.data.type_.clone(),
			})
		}
	}

	pub fn require_datatype(
		&self,
		context: &crate::Context<M>,
	) -> Result<&ty::datatype::Definition<M>, NodeTypeInvalid<M>> {
		if self.has_type(context, ty::SubClass::DataType) {
			Ok(self.as_datatype())
		} else {
			Err(NodeTypeInvalid {
				id: self.data.id,
				expected: ty::SubClass::DataType.into(),
				found: self.data.type_.clone(),
			})
		}
	}

	pub fn require_datatype_id(
		&self,
		context: &crate::Context<M>,
	) -> Result<treeldr::TId<treeldr::ty::DataType<M>>, NodeTypeInvalid<M>> {
		if self.has_type(context, ty::SubClass::DataType) {
			Ok(treeldr::TId::new(self.data.id))
		} else {
			Err(NodeTypeInvalid {
				id: self.data.id,
				expected: ty::SubClass::DataType.into(),
				found: self.data.type_.clone(),
			})
		}
	}

	pub fn require_restriction(
		&self,
		context: &crate::Context<M>,
	) -> Result<&ty::restriction::Definition<M>, NodeTypeInvalid<M>> {
		if self.has_type(context, ty::SubClass::Restriction) {
			Ok(self.as_restriction())
		} else {
			Err(NodeTypeInvalid {
				id: self.data.id,
				expected: ty::SubClass::Restriction.into(),
				found: self.data.type_.clone(),
			})
		}
	}

	pub fn require_restriction_id(
		&self,
		context: &crate::Context<M>,
	) -> Result<treeldr::TId<treeldr::ty::Restriction>, NodeTypeInvalid<M>> {
		if self.has_type(context, Type::LayoutRestriction) {
			Ok(treeldr::TId::new(self.data.id))
		} else {
			Err(NodeTypeInvalid {
				id: self.data.id,
				expected: Type::LayoutRestriction.into(),
				found: self.data.type_.clone(),
			})
		}
	}

	pub fn require_datatype_restriction(
		&self,
		context: &crate::Context<M>,
	) -> Result<&ty::datatype::restriction::Definition<M>, NodeTypeInvalid<M>> {
		if self.has_type(context, Type::DatatypeRestriction) {
			Ok(self.as_datatype_restriction())
		} else {
			Err(NodeTypeInvalid {
				id: self.data.id,
				expected: Type::DatatypeRestriction.into(),
				found: self.data.type_.clone(),
			})
		}
	}

	pub fn require_datatype_restriction_id(
		&self,
		context: &crate::Context<M>,
	) -> Result<treeldr::TId<treeldr::ty::data::Restriction>, NodeTypeInvalid<M>> {
		if self.has_type(context, Type::DatatypeRestriction) {
			Ok(treeldr::TId::new(self.data.id))
		} else {
			Err(NodeTypeInvalid {
				id: self.data.id,
				expected: Type::DatatypeRestriction.into(),
				found: self.data.type_.clone(),
			})
		}
	}

	pub fn require_property(
		&self,
		context: &crate::Context<M>,
	) -> Result<&prop::Definition<M>, NodeTypeInvalid<M>> {
		if self.has_type(context, Type::Property(None)) {
			Ok(self.as_property())
		} else {
			Err(NodeTypeInvalid {
				id: self.data.id,
				expected: Type::Property(None).into(),
				found: self.data.type_.clone(),
			})
		}
	}

	pub fn require_property_id(
		&self,
		context: &crate::Context<M>,
	) -> Result<treeldr::TId<treeldr::Property>, NodeTypeInvalid<M>> {
		if self.has_type(context, Type::Property(None)) {
			Ok(treeldr::TId::new(self.data.id))
		} else {
			Err(NodeTypeInvalid {
				id: self.data.id,
				expected: Type::Property(None).into(),
				found: self.data.type_.clone(),
			})
		}
	}

	pub fn require_layout(
		&self,
		context: &crate::Context<M>,
	) -> Result<&layout::Definition<M>, NodeTypeInvalid<M>> {
		if self.has_type(context, component::Type::Layout) {
			Ok(self.as_layout())
		} else {
			Err(NodeTypeInvalid {
				id: self.data.id,
				expected: component::Type::Layout.into(),
				found: self.data.type_.clone(),
			})
		}
	}

	pub fn require_layout_id(
		&self,
		context: &crate::Context<M>,
	) -> Result<treeldr::TId<treeldr::Layout>, NodeTypeInvalid<M>> {
		if self.has_type(context, component::Type::Layout) {
			Ok(treeldr::TId::new(self.data.id))
		} else {
			Err(NodeTypeInvalid {
				id: self.data.id,
				expected: component::Type::Layout.into(),
				found: self.data.type_.clone(),
			})
		}
	}

	pub fn require_layout_field(
		&self,
		context: &crate::Context<M>,
	) -> Result<&layout::field::Definition<M>, NodeTypeInvalid<M>> {
		if self.has_type(context, component::formatted::Type::LayoutField) {
			Ok(self.as_layout_field())
		} else {
			Err(NodeTypeInvalid {
				id: self.data.id,
				expected: component::formatted::Type::LayoutField.into(),
				found: self.data.type_.clone(),
			})
		}
	}

	pub fn require_layout_field_id(
		&self,
		context: &crate::Context<M>,
	) -> Result<treeldr::TId<treeldr::layout::Field>, NodeTypeInvalid<M>> {
		if self.has_type(context, component::formatted::Type::LayoutField) {
			Ok(treeldr::TId::new(self.data.id))
		} else {
			Err(NodeTypeInvalid {
				id: self.data.id,
				expected: component::formatted::Type::LayoutField.into(),
				found: self.data.type_.clone(),
			})
		}
	}

	pub fn require_layout_variant(
		&self,
		context: &crate::Context<M>,
	) -> Result<&layout::variant::Definition, NodeTypeInvalid<M>> {
		if self.has_type(context, component::formatted::Type::LayoutVariant) {
			Ok(self.as_layout_variant())
		} else {
			Err(NodeTypeInvalid {
				id: self.data.id,
				expected: component::formatted::Type::LayoutVariant.into(),
				found: self.data.type_.clone(),
			})
		}
	}

	pub fn require_layout_variant_id(
		&self,
		context: &crate::Context<M>,
	) -> Result<treeldr::TId<treeldr::layout::Variant>, NodeTypeInvalid<M>> {
		if self.has_type(context, component::formatted::Type::LayoutVariant) {
			Ok(treeldr::TId::new(self.data.id))
		} else {
			Err(NodeTypeInvalid {
				id: self.data.id,
				expected: component::formatted::Type::LayoutVariant.into(),
				found: self.data.type_.clone(),
			})
		}
	}

	pub fn require_layout_restriction(
		&self,
		context: &crate::Context<M>,
	) -> Result<&layout::restriction::Definition<M>, NodeTypeInvalid<M>> {
		if self.has_type(context, Type::LayoutRestriction) {
			Ok(self.as_layout_restriction())
		} else {
			Err(NodeTypeInvalid {
				id: self.data.id,
				expected: Type::LayoutRestriction.into(),
				found: self.data.type_.clone(),
			})
		}
	}

	pub fn require_layout_restriction_id(
		&self,
		context: &crate::Context<M>,
	) -> Result<treeldr::TId<treeldr::layout::ContainerRestriction>, NodeTypeInvalid<M>> {
		if self.has_type(context, Type::LayoutRestriction) {
			Ok(treeldr::TId::new(self.data.id))
		} else {
			Err(NodeTypeInvalid {
				id: self.data.id,
				expected: Type::LayoutRestriction.into(),
				found: self.data.type_.clone(),
			})
		}
	}

	pub fn require_list(
		&self,
		context: &crate::Context<M>,
	) -> Result<&list::Definition<M>, NodeTypeInvalid<M>> {
		if self.has_type(context, Type::List) {
			Ok(self.as_list())
		} else {
			Err(NodeTypeInvalid {
				id: self.data.id,
				expected: Type::List.into(),
				found: self.data.type_.clone(),
			})
		}
	}

	pub(crate) fn build(
		&self,
		context: &crate::Context<M>,
	) -> Result<Option<treeldr::node::Definition<M>>, Error<M>>
	where
		M: Merge,
	{
		let type_ = self.data.type_.try_mapped(|_, Meta(ty, m)| {
			context
				.require_type_id(ty.id().id())
				.map(|ty| Meta(ty, m.clone()))
		}).map_err(|(Meta(e, m), _)| e.at_node_property(self.data.id, Property::Type, m.clone()))?;

		let doc = treeldr::Documentation::from_comments(self.data.comment.map(From::from));

		let data = treeldr::node::Data {
			id: self.data.id,
			metadata: self.data.metadata.clone(),
			type_,
			label: self.data.label.clone(),
			comment: doc,
		};

		let ty: MetaOption<treeldr::ty::Definition<M>, M> = self
			.data
			.type_metadata(context, Type::Class(None))
			.map(|meta| self.ty.build(context, &data, meta.clone()))
			.transpose()?
			.into();

		let property: MetaOption<treeldr::prop::Definition<M>, M> = self
			.data
			.type_metadata(context, Type::Property(None))
			.map(|meta| self.property.build(context, &data, meta.clone()))
			.transpose()?
			.into();

		let component: MetaOption<treeldr::component::Definition<M>, M> = self
			.data
			.type_metadata(context, Type::Component(None))
			.map(|meta| self.component.build(context, &data, meta.clone()))
			.transpose()?
			.into();

		if ty.is_none()
			&& property.is_none()
			&& component.is_none()
			&& self
				.data
				.type_
				.contains(&crate::Type::Resource(Some(Type::List)))
		{
			return Ok(None);
		}

		Ok(Some(treeldr::node::Definition::new(
			data, ty, property, component,
		)))
	}
}

pub enum ClassBindingRef<'a> {
	Type(Option<Id>, crate::Type),
	Label(Option<Id>, &'a str),
	Comment(Option<Id>, &'a str),
}

impl<'a> ClassBindingRef<'a> {
	pub fn into_binding_ref<M>(self) -> BindingRef<'a, M> {
		match self {
			Self::Type(_, t) => BindingRef::Type(t),
			Self::Label(_, l) => BindingRef::Label(l),
			Self::Comment(_, c) => BindingRef::Comment(c),
		}
	}
}

pub struct ClassBindings<'a, M> {
	type_: property_values::non_functional::Iter<'a, crate::Type, M>,
	label: property_values::non_functional::Iter<'a, String, M>,
	comment: property_values::non_functional::Iter<'a, String, M>,
}

impl<'a, M> Iterator for ClassBindings<'a, M> {
	type Item = Meta<ClassBindingRef<'a>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.type_
			.next()
			.map(|m| m.into_cloned_class_binding(ClassBindingRef::Type))
			.or_else(|| {
				self.label
					.next()
					.map(|m| m.into_deref_class_binding(ClassBindingRef::Label))
					.or_else(|| {
						self.comment
							.next()
							.map(|m| m.into_deref_class_binding(ClassBindingRef::Comment))
					})
			})
	}
}

pub enum BindingValueRef<'a, M> {
	Type(crate::Type),
	Id(Id),
	Boolean(bool),
	String(&'a str),
	Name(&'a Name),
	Object(&'a Object<M>),
	Numeric(&'a value::Numeric),
	Integer(&'a value::Integer),
	NonNegativeInteger(&'a value::NonNegativeInteger),
	RegExp(&'a RegExp),
}

impl<'a, M> BindingValueRef<'a, M> {
	pub fn into_id(self) -> Option<Id> {
		match self {
			Self::Type(ty) => Some(ty.id().id()),
			Self::Id(id) => Some(id),
			Self::Object(obj) => obj.as_id(),
			_ => None,
		}
	}
}

pub enum BindingRef<'a, M> {
	Type(crate::Type),
	Label(&'a str),
	Comment(&'a str),
	Class(crate::ty::BindingRef<'a>),
	DatatypeRestriction(crate::ty::datatype::restriction::BindingRef<'a>),
	Property(crate::prop::Binding),
	Component(crate::component::BindingRef<'a>),
	LayoutRestriction(crate::layout::restriction::BindingRef<'a>),
	List(crate::list::BindingRef<'a, M>),
}

impl<'a, M> BindingRef<'a, M> {
	pub fn resource_property(&self) -> Property {
		match self {
			Self::Type(_) => Property::Type,
			Self::Label(_) => Property::Label,
			Self::Comment(_) => Property::Comment,
			Self::Class(b) => Property::Class(b.property()),
			Self::DatatypeRestriction(b) => Property::DatatypeRestriction(b.property()),
			Self::Property(b) => Property::Property(b.property()),
			Self::Component(b) => Property::Component(b.property()),
			Self::LayoutRestriction(b) => Property::LayoutRestriction(b.property()),
			Self::List(b) => Property::List(b.property()),
		}
	}

	pub fn property(&self) -> crate::Property {
		crate::Property::Resource(self.resource_property())
	}

	pub fn value(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::Type(v) => BindingValueRef::Type(*v),
			Self::Label(v) => BindingValueRef::String(v),
			Self::Comment(v) => BindingValueRef::String(v),
			Self::Class(b) => b.value(),
			Self::DatatypeRestriction(b) => b.value(),
			Self::Property(b) => b.value(),
			Self::Component(b) => b.value(),
			Self::LayoutRestriction(b) => b.value(),
			Self::List(b) => b.value(),
		}
	}
}

/// Iterator over the bindings of a given node.
pub struct Bindings<'a, M> {
	data: ClassBindings<'a, M>,
	class: crate::ty::Bindings<'a, M>,
	datatype_restriction: crate::ty::datatype::restriction::Bindings<'a, M>,
	property: crate::prop::Bindings<'a, M>,
	component: crate::component::Bindings<'a, M>,
	layout_restriction: crate::layout::restriction::Bindings<'a, M>,
	list: crate::list::Bindings<'a, M>,
}

impl<'a, M> Iterator for Bindings<'a, M> {
	type Item = Meta<BindingRef<'a, M>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.data
			.next()
			.map(|m| m.map(ClassBindingRef::into_binding_ref))
			.or_else(|| {
				self.class
					.next()
					.map(|m| m.map(BindingRef::Class))
					.or_else(|| {
						self.datatype_restriction
							.next()
							.map(|m| m.map(BindingRef::DatatypeRestriction))
							.or_else(|| {
								self.property
									.next()
									.map(|m| m.map(BindingRef::Property))
									.or_else(|| {
										self.component
											.next()
											.map(|m| m.map(BindingRef::Component))
											.or_else(|| {
												self.layout_restriction
													.next()
													.map(|m| m.map(BindingRef::LayoutRestriction))
													.or_else(|| {
														self.list
															.next()
															.map(|m| m.map(BindingRef::List))
													})
											})
									})
							})
					})
			})
	}
}
