use crate::{error::NodeTypeInvalid, layout, list, prop, ty, resource, component, Single};
use treeldr::{Id, Multiple, Name};

pub use treeldr::Type;

pub mod property;

pub use property::{Bindings, Property};

#[derive(Clone)]
pub struct Node<M> {
	data: resource::Data<M>,
	ty: ty::Definition<M>,
	datatype_restriction: ty::datatype::restriction::Definition<M>,
	property: prop::Definition<M>,
	component: component::Definition<M>,
	layout_restriction: layout::restriction::Definition<M>,
	list: list::Definition<M>
}

impl<M> Node<M> {
	pub fn new(id: Id, metadata: M) -> Self {
		Self {
			data: resource::Data::new(id, metadata),
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

	pub fn has_type(&self, id: impl Into<Type>) -> bool {
		self.data.type_.contains(&id.into())
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

	pub fn name(&self) -> &Single<Name, M> {
		&self.data.name
	}

	pub fn name_mut(&mut self) -> &mut Single<Name, M> {
		&mut self.data.name
	}

	pub fn format(&self) -> &Single<Id, M> {
		&self.data.format
	}

	pub fn format_mut(&mut self) -> &mut Single<Id, M> {
		&mut self.data.format
	}
	
	pub fn as_resource(&self) -> &resource::Data<M> {
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

impl<M: Clone> Node<M> {
	pub fn require_type(&self) -> Result<&ty::Definition<M>, NodeTypeInvalid<M>> {
		if self.has_type(Type::Class(None)) {
			Ok(self.as_type())
		} else {
			Err(NodeTypeInvalid { id: self.data.id, expected: Type::Class(None), found: self.data.type_.clone() })
		}
	}

	pub fn require_type_id(&self) -> Result<treeldr::TId<Type>, NodeTypeInvalid<M>> {
		if self.has_type(Type::Class(None)) {
			Ok(treeldr::TId::new(self.data.id))
		} else {
			Err(NodeTypeInvalid { id: self.data.id, expected: Type::Class(None), found: self.data.type_.clone() })
		}
	}

	pub fn require_datatype(&self) -> Result<&ty::datatype::Definition<M>, NodeTypeInvalid<M>> {
		if self.has_type(ty::SubClass::DataType) {
			Ok(self.as_datatype())
		} else {
			Err(NodeTypeInvalid { id: self.data.id, expected: ty::SubClass::DataType.into(), found: self.data.type_.clone() })
		}
	}

	pub fn require_datatype_id(&self) -> Result<treeldr::TId<treeldr::ty::DataType>, NodeTypeInvalid<M>> {
		if self.has_type(ty::SubClass::DataType) {
			Ok(treeldr::TId::new(self.data.id))
		} else {
			Err(NodeTypeInvalid { id: self.data.id, expected: ty::SubClass::DataType.into(), found: self.data.type_.clone() })
		}
	}

	pub fn require_restriction(&self) -> Result<&ty::restriction::Definition<M>, NodeTypeInvalid<M>> {
		if self.has_type(ty::SubClass::Restriction) {
			Ok(self.as_restriction())
		} else {
			Err(NodeTypeInvalid { id: self.data.id, expected: ty::SubClass::Restriction.into(), found: self.data.type_.clone() })
		}
	}

	pub fn require_restriction_id(&self) -> Result<treeldr::TId<treeldr::ty::Restriction>, NodeTypeInvalid<M>> {
		if self.has_type(Type::LayoutRestriction) {
			Ok(treeldr::TId::new(self.data.id))
		} else {
			Err(NodeTypeInvalid { id: self.data.id, expected: Type::LayoutRestriction, found: self.data.type_.clone() })
		}
	}

	pub fn require_datatype_restriction(&self) -> Result<&ty::datatype::restriction::Definition<M>, NodeTypeInvalid<M>> {
		if self.has_type(Type::DatatypeRestriction) {
			Ok(self.as_datatype_restriction())
		} else {
			Err(NodeTypeInvalid { id: self.data.id, expected: Type::DatatypeRestriction, found: self.data.type_.clone() })
		}
	}

	pub fn require_datatype_restriction_id(&self) -> Result<treeldr::TId<treeldr::ty::data::Restriction>, NodeTypeInvalid<M>> {
		if self.has_type(Type::DatatypeRestriction) {
			Ok(treeldr::TId::new(self.data.id))
		} else {
			Err(NodeTypeInvalid { id: self.data.id, expected: Type::DatatypeRestriction, found: self.data.type_.clone() })
		}
	}

	pub fn require_property(&self) -> Result<&prop::Definition<M>, NodeTypeInvalid<M>> {
		if self.has_type(Type::Property) {
			Ok(self.as_property())
		} else {
			Err(NodeTypeInvalid { id: self.data.id, expected: Type::Property, found: self.data.type_.clone() })
		}
	}

	pub fn require_property_id(&self) -> Result<treeldr::TId<Property>, NodeTypeInvalid<M>> {
		if self.has_type(Type::Property) {
			Ok(treeldr::TId::new(self.data.id))
		} else {
			Err(NodeTypeInvalid { id: self.data.id, expected: Type::Property, found: self.data.type_.clone() })
		}
	}

	pub fn require_layout(&self) -> Result<&layout::Definition<M>, NodeTypeInvalid<M>> {
		if self.has_type(component::Type::Layout) {
			Ok(self.as_layout())
		} else {
			Err(NodeTypeInvalid { id: self.data.id, expected: component::Type::Layout.into(), found: self.data.type_.clone() })
		}
	}

	pub fn require_layout_id(&self) -> Result<treeldr::TId<treeldr::Layout>, NodeTypeInvalid<M>> {
		if self.has_type(component::Type::Layout) {
			Ok(treeldr::TId::new(self.data.id))
		} else {
			Err(NodeTypeInvalid { id: self.data.id, expected: component::Type::Layout.into(), found: self.data.type_.clone() })
		}
	}

	pub fn require_layout_field(&self) -> Result<&layout::field::Definition<M>, NodeTypeInvalid<M>> {
		if self.has_type(component::formatted::Type::LayoutField) {
			Ok(self.as_layout_field())
		} else {
			Err(NodeTypeInvalid { id: self.data.id, expected: component::formatted::Type::LayoutField.into(), found: self.data.type_.clone() })
		}
	}

	pub fn require_layout_field_id(&self) -> Result<treeldr::TId<treeldr::layout::Field>, NodeTypeInvalid<M>> {
		if self.has_type(component::formatted::Type::LayoutField) {
			Ok(treeldr::TId::new(self.data.id))
		} else {
			Err(NodeTypeInvalid { id: self.data.id, expected: component::formatted::Type::LayoutField.into(), found: self.data.type_.clone() })
		}
	}

	pub fn require_layout_variant(&self) -> Result<&layout::variant::Definition, NodeTypeInvalid<M>> {
		if self.has_type(component::formatted::Type::LayoutVariant) {
			Ok(self.as_layout_variant())
		} else {
			Err(NodeTypeInvalid { id: self.data.id, expected: component::formatted::Type::LayoutVariant.into(), found: self.data.type_.clone() })
		}
	}

	pub fn require_layout_variant_id(&self) -> Result<treeldr::TId<treeldr::layout::Variant>, NodeTypeInvalid<M>> {
		if self.has_type(component::formatted::Type::LayoutVariant) {
			Ok(treeldr::TId::new(self.data.id))
		} else {
			Err(NodeTypeInvalid { id: self.data.id, expected: component::formatted::Type::LayoutVariant.into(), found: self.data.type_.clone() })
		}
	}

	pub fn require_layout_restriction(&self) -> Result<&layout::restriction::Definition<M>, NodeTypeInvalid<M>> {
		if self.has_type(Type::LayoutRestriction) {
			Ok(self.as_layout_restriction())
		} else {
			Err(NodeTypeInvalid { id: self.data.id, expected: Type::LayoutRestriction, found: self.data.type_.clone() })
		}
	}

	pub fn require_layout_restriction_id(&self) -> Result<treeldr::TId<treeldr::layout::Restriction>, NodeTypeInvalid<M>> {
		if self.has_type(Type::LayoutRestriction) {
			Ok(treeldr::TId::new(self.data.id))
		} else {
			Err(NodeTypeInvalid { id: self.data.id, expected: Type::LayoutRestriction, found: self.data.type_.clone() })
		}
	}

	pub fn require_list(&self) -> Result<&list::Definition<M>, NodeTypeInvalid<M>> {
		if self.has_type(Type::List) {
			Ok(self.as_list())
		} else {
			Err(NodeTypeInvalid { id: self.data.id, expected: Type::List, found: self.data.type_.clone() })
		}
	}
}