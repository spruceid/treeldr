use locspan::Meta;
use treeldr::{Id, Type, metadata::Merge};

use crate::{Property, Multiple, multiple, ty, prop, component, layout, list, error::NodeTypeInvalid, Error, context::HasType};

#[derive(Debug, Clone)]
pub struct AnonymousData<M> {
	pub type_: Multiple<Type, M>,
	pub label: Multiple<String, M>,
	pub comment: Multiple<String, M>
}

#[derive(Debug, Clone)]
pub struct Data<M> {
	pub id: Id,
	pub metadata: M,
	pub type_: Multiple<Type, M>,
	pub label: Multiple<String, M>,
	pub comment: Multiple<String, M>
}

impl<M> Data<M> {
	pub fn new(id: Id, metadata: M) -> Self {
		Self {
			id,
			metadata,
			type_: Multiple::default(),
			label: Multiple::default(),
			comment: Multiple::default()
		}
	}

	pub fn clone_anonymous(&self) -> AnonymousData<M> where M: Clone {
		AnonymousData {
			type_: self.type_.clone(),
			label: self.label.clone(),
			comment: self.comment.clone()
		}
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
	list: list::Definition<M>
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
			list: list::Definition::new()
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

	pub fn type_(&self) -> &Multiple<Type, M> {
		&self.data.type_
	}

	pub fn type_mut(&mut self) -> &mut Multiple<Type, M> {
		&mut self.data.type_
	}

	pub fn has_type(&self, context: &crate::Context<M>, type_: impl Into<Type>) -> bool {
		self.data.has_type(context, type_)
	}

	pub fn label(&self) -> &Multiple<String, M> {
		&self.data.label
	}

	pub fn label_mut(&mut self) -> &mut Multiple<String, M> {
		&mut self.data.label
	}

	pub fn comment(&self) -> &Multiple<String, M> {
		&self.data.comment
	}

	pub fn comment_mut(&mut self) -> &mut Multiple<String, M> {
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

	pub fn as_layout_variant_mut(
		&mut self,
	) -> &mut layout::variant::Definition {
		self.component.as_layout_variant_mut()
	}

	pub fn as_list_mut(&mut self) -> &mut list::Definition<M> {
		&mut self.list
	}
}

impl<M: Clone> Definition<M> {
	pub fn require_type(&self, context: &crate::Context<M>) -> Result<&ty::Definition<M>, NodeTypeInvalid<M>> {
		if self.has_type(context, Type::Class(None)) {
			Ok(self.as_type())
		} else {
			Err(NodeTypeInvalid { id: self.data.id, expected: Type::Class(None), found: self.data.type_.clone() })
		}
	}

	pub fn require_type_id(&self, context: &crate::Context<M>) -> Result<treeldr::TId<Type>, NodeTypeInvalid<M>> {
		if self.has_type(context, Type::Class(None)) {
			Ok(treeldr::TId::new(self.data.id))
		} else {
			Err(NodeTypeInvalid { id: self.data.id, expected: Type::Class(None), found: self.data.type_.clone() })
		}
	}

	pub fn require_datatype(&self, context: &crate::Context<M>) -> Result<&ty::datatype::Definition<M>, NodeTypeInvalid<M>> {
		if self.has_type(context, ty::SubClass::DataType) {
			Ok(self.as_datatype())
		} else {
			Err(NodeTypeInvalid { id: self.data.id, expected: ty::SubClass::DataType.into(), found: self.data.type_.clone() })
		}
	}

	pub fn require_datatype_id(&self, context: &crate::Context<M>) -> Result<treeldr::TId<treeldr::ty::DataType>, NodeTypeInvalid<M>> {
		if self.has_type(context, ty::SubClass::DataType) {
			Ok(treeldr::TId::new(self.data.id))
		} else {
			Err(NodeTypeInvalid { id: self.data.id, expected: ty::SubClass::DataType.into(), found: self.data.type_.clone() })
		}
	}

	pub fn require_restriction(&self, context: &crate::Context<M>) -> Result<&ty::restriction::Definition<M>, NodeTypeInvalid<M>> {
		if self.has_type(context, ty::SubClass::Restriction) {
			Ok(self.as_restriction())
		} else {
			Err(NodeTypeInvalid { id: self.data.id, expected: ty::SubClass::Restriction.into(), found: self.data.type_.clone() })
		}
	}

	pub fn require_restriction_id(&self, context: &crate::Context<M>) -> Result<treeldr::TId<treeldr::ty::Restriction>, NodeTypeInvalid<M>> {
		if self.has_type(context, Type::LayoutRestriction) {
			Ok(treeldr::TId::new(self.data.id))
		} else {
			Err(NodeTypeInvalid { id: self.data.id, expected: Type::LayoutRestriction, found: self.data.type_.clone() })
		}
	}

	pub fn require_datatype_restriction(&self, context: &crate::Context<M>) -> Result<&ty::datatype::restriction::Definition<M>, NodeTypeInvalid<M>> {
		if self.has_type(context, Type::DatatypeRestriction) {
			Ok(self.as_datatype_restriction())
		} else {
			Err(NodeTypeInvalid { id: self.data.id, expected: Type::DatatypeRestriction, found: self.data.type_.clone() })
		}
	}

	pub fn require_datatype_restriction_id(&self, context: &crate::Context<M>) -> Result<treeldr::TId<treeldr::ty::data::Restriction>, NodeTypeInvalid<M>> {
		if self.has_type(context, Type::DatatypeRestriction) {
			Ok(treeldr::TId::new(self.data.id))
		} else {
			Err(NodeTypeInvalid { id: self.data.id, expected: Type::DatatypeRestriction, found: self.data.type_.clone() })
		}
	}

	pub fn require_property(&self, context: &crate::Context<M>) -> Result<&prop::Definition<M>, NodeTypeInvalid<M>> {
		if self.has_type(context, Type::Property(None)) {
			Ok(self.as_property())
		} else {
			Err(NodeTypeInvalid { id: self.data.id, expected: Type::Property(None), found: self.data.type_.clone() })
		}
	}

	pub fn require_property_id(&self, context: &crate::Context<M>) -> Result<treeldr::TId<Property>, NodeTypeInvalid<M>> {
		if self.has_type(context, Type::Property(None)) {
			Ok(treeldr::TId::new(self.data.id))
		} else {
			Err(NodeTypeInvalid { id: self.data.id, expected: Type::Property(None), found: self.data.type_.clone() })
		}
	}

	pub fn require_layout(&self, context: &crate::Context<M>) -> Result<&layout::Definition<M>, NodeTypeInvalid<M>> {
		if self.has_type(context, component::Type::Layout) {
			Ok(self.as_layout())
		} else {
			Err(NodeTypeInvalid { id: self.data.id, expected: component::Type::Layout.into(), found: self.data.type_.clone() })
		}
	}

	pub fn require_layout_id(&self, context: &crate::Context<M>) -> Result<treeldr::TId<treeldr::Layout>, NodeTypeInvalid<M>> {
		if self.has_type(context, component::Type::Layout) {
			Ok(treeldr::TId::new(self.data.id))
		} else {
			Err(NodeTypeInvalid { id: self.data.id, expected: component::Type::Layout.into(), found: self.data.type_.clone() })
		}
	}

	pub fn require_layout_field(&self, context: &crate::Context<M>) -> Result<&layout::field::Definition<M>, NodeTypeInvalid<M>> {
		if self.has_type(context, component::formatted::Type::LayoutField) {
			Ok(self.as_layout_field())
		} else {
			Err(NodeTypeInvalid { id: self.data.id, expected: component::formatted::Type::LayoutField.into(), found: self.data.type_.clone() })
		}
	}

	pub fn require_layout_field_id(&self, context: &crate::Context<M>) -> Result<treeldr::TId<treeldr::layout::Field>, NodeTypeInvalid<M>> {
		if self.has_type(context, component::formatted::Type::LayoutField) {
			Ok(treeldr::TId::new(self.data.id))
		} else {
			Err(NodeTypeInvalid { id: self.data.id, expected: component::formatted::Type::LayoutField.into(), found: self.data.type_.clone() })
		}
	}

	pub fn require_layout_variant(&self, context: &crate::Context<M>) -> Result<&layout::variant::Definition, NodeTypeInvalid<M>> {
		if self.has_type(context, component::formatted::Type::LayoutVariant) {
			Ok(self.as_layout_variant())
		} else {
			Err(NodeTypeInvalid { id: self.data.id, expected: component::formatted::Type::LayoutVariant.into(), found: self.data.type_.clone() })
		}
	}

	pub fn require_layout_variant_id(&self, context: &crate::Context<M>) -> Result<treeldr::TId<treeldr::layout::Variant>, NodeTypeInvalid<M>> {
		if self.has_type(context, component::formatted::Type::LayoutVariant) {
			Ok(treeldr::TId::new(self.data.id))
		} else {
			Err(NodeTypeInvalid { id: self.data.id, expected: component::formatted::Type::LayoutVariant.into(), found: self.data.type_.clone() })
		}
	}

	pub fn require_layout_restriction(&self, context: &crate::Context<M>) -> Result<&layout::restriction::Definition<M>, NodeTypeInvalid<M>> {
		if self.has_type(context, Type::LayoutRestriction) {
			Ok(self.as_layout_restriction())
		} else {
			Err(NodeTypeInvalid { id: self.data.id, expected: Type::LayoutRestriction, found: self.data.type_.clone() })
		}
	}

	pub fn require_layout_restriction_id(&self, context: &crate::Context<M>) -> Result<treeldr::TId<treeldr::layout::Restriction>, NodeTypeInvalid<M>> {
		if self.has_type(context, Type::LayoutRestriction) {
			Ok(treeldr::TId::new(self.data.id))
		} else {
			Err(NodeTypeInvalid { id: self.data.id, expected: Type::LayoutRestriction, found: self.data.type_.clone() })
		}
	}

	pub fn require_list(&self, context: &crate::Context<M>) -> Result<&list::Definition<M>, NodeTypeInvalid<M>> {
		if self.has_type(context, Type::List) {
			Ok(self.as_list())
		} else {
			Err(NodeTypeInvalid { id: self.data.id, expected: Type::List, found: self.data.type_.clone() })
		}
	}

	pub(crate) fn build(
		&self,
		context: &crate::Context<M>
	) -> Result<treeldr::node::Definition<M>, Error<M>> where M: Merge {
		let data = treeldr::node::Data {
			id: self.data.id,
			metadata: self.data.metadata.clone(),
			type_: self.data.type_.clone(),
			label: self.data.label.clone(),
			comment: self.data.comment.clone()
		};

		let ty = self.data.type_metadata(context, Type::Class(None)).map(|meta| {
			self.ty.build(context, &data, meta.clone())
		}).transpose()?.into();

		let property = self.data.type_metadata(context, Type::Property(None)).map(|meta| {
			self.property.build(context, &data, meta.clone())
		}).transpose()?.into();

		let component = self.data.type_metadata(context, Type::Component(None)).map(|meta| {
			self.component.build(context, &data, meta.clone())
		}).transpose()?.into();

		Ok(treeldr::node::Definition::new(data, ty, property, component))
	}
}

pub enum ClassBindingRef<'a> {
	Type(Id),
	Label(&'a str),
	Comment(&'a str)
}

impl<'a> ClassBindingRef<'a> {
	pub fn into_binding_ref<M>(self) -> BindingRef<'a, M> {
		match self {
			Self::Type(t) => BindingRef::Type(t),
			Self::Label(l) => BindingRef::Label(l),
			Self::Comment(c) => BindingRef::Comment(c)
		}
	}
}

pub struct ClassBindings<'a, M> {
	type_: multiple::Iter<'a, Id, M>,
	label: multiple::Iter<'a, String, M>,
	comment: multiple::Iter<'a, String, M>
}

impl<'a, M> Iterator for ClassBindings<'a, M> {
	type Item = Meta<ClassBindingRef<'a>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.type_
			.next()
			.map(Meta::into_cloned_value)
			.map(|m| m.map(ClassBindingRef::Type))
			.or_else(|| {
				self.label
					.next()
					.map(|v| v.map(String::as_str))
					.map(|m| m.map(ClassBindingRef::Label))
					.or_else(|| {
						self.comment
							.next()
							.map(|v| v.map(String::as_str))
							.map(|m| m.map(ClassBindingRef::Comment))
					})
			})
	}
}

pub enum BindingRef<'a, M> {
	Type(Id),
	Label(&'a str),
	Comment(&'a str),
	Class(crate::ty::Binding),
	DatatypeRestriction(crate::ty::datatype::restriction::BindingRef<'a>),
	Property(crate::prop::Binding),
	Component(crate::component::BindingRef<'a>),
	LayoutRestriction(crate::layout::restriction::BindingRef<'a>),
	List(crate::list::BindingRef<'a, M>)
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