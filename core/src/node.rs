use crate::{error, layout, prop, ty, Documentation, Error, Id, MetaOption};
use locspan::Meta;
use shelves::Ref;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Type {
	Type,
	Property,
	Layout,
	LayoutField,
	LayoutVariant,
	List,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Types {
	pub ty: bool,
	pub property: bool,
	pub layout: bool,
	pub layout_field: bool,
	pub layout_variant: bool,
	pub list: bool,
}

impl Types {
	pub fn includes(&self, ty: Type) -> bool {
		match ty {
			Type::Type => self.ty,
			Type::Property => self.property,
			Type::Layout => self.layout,
			Type::LayoutField => self.layout_field,
			Type::LayoutVariant => self.layout_variant,
			Type::List => self.list,
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct TypesMetadata<M> {
	pub ty: Option<M>,
	pub property: Option<M>,
	pub layout: Option<M>,
	pub layout_field: Option<M>,
	pub layout_variant: Option<M>,
	pub list: Option<M>,
}

impl<M> TypesMetadata<M> {
	pub fn is_empty(&self) -> bool {
		self.ty.is_none() && self.property.is_none() && self.layout.is_none()
	}

	pub fn includes(&self, ty: Type) -> Option<&M> {
		match ty {
			Type::Type => self.ty.as_ref(),
			Type::Property => self.property.as_ref(),
			Type::Layout => self.layout.as_ref(),
			Type::LayoutField => self.layout_field.as_ref(),
			Type::LayoutVariant => self.layout_variant.as_ref(),
			Type::List => self.list.as_ref(),
		}
	}

	pub fn iter(&self) -> TypesMetadataIter<M> {
		TypesMetadataIter {
			ty: self.ty.as_ref(),
			property: self.property.as_ref(),
			layout: self.layout.as_ref(),
			layout_field: self.layout_field.as_ref(),
			layout_variant: self.layout_variant.as_ref(),
			list: self.list.as_ref(),
		}
	}
}

impl<'a, M: Clone> TypesMetadata<&'a M> {
	pub fn cloned(&self) -> TypesMetadata<M> {
		TypesMetadata {
			ty: self.ty.cloned(),
			property: self.property.cloned(),
			layout: self.layout.cloned(),
			layout_field: self.layout_field.cloned(),
			layout_variant: self.layout_variant.cloned(),
			list: self.list.cloned(),
		}
	}
}

impl<'a, M> IntoIterator for &'a TypesMetadata<M> {
	type Item = Meta<Type, &'a M>;
	type IntoIter = TypesMetadataIter<'a, M>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

pub struct TypesMetadataIter<'a, M> {
	ty: Option<&'a M>,
	property: Option<&'a M>,
	layout: Option<&'a M>,
	layout_field: Option<&'a M>,
	layout_variant: Option<&'a M>,
	list: Option<&'a M>,
}

impl<'a, M> Iterator for TypesMetadataIter<'a, M> {
	type Item = Meta<Type, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		match self.ty.take() {
			Some(metadata) => Some(Meta(Type::Type, metadata)),
			None => match self.property.take() {
				Some(metadata) => Some(Meta(Type::Property, metadata)),
				None => match self.layout.take() {
					Some(metadata) => Some(Meta(Type::Layout, metadata)),
					None => match self.layout_field.take() {
						Some(metadata) => Some(Meta(Type::LayoutField, metadata)),
						None => match self.layout_variant.take() {
							Some(metadata) => Some(Meta(Type::LayoutVariant, metadata)),
							None => self.list.take().map(|metadata| Meta(Type::List, metadata)),
						},
					},
				},
			},
		}
	}
}

#[derive(Debug)]
pub struct Node<M> {
	id: Id,
	label: Option<String>,
	ty: MetaOption<Ref<ty::Definition<M>>, M>,
	property: MetaOption<Ref<prop::Definition<M>>, M>,
	layout: MetaOption<Ref<layout::Definition<M>>, M>,
	doc: Documentation,
}

impl<M> Node<M> {
	pub fn new(id: Id) -> Self {
		Self {
			id,
			label: None,
			ty: MetaOption::default(),
			property: MetaOption::default(),
			layout: MetaOption::default(),
			doc: Documentation::default(),
		}
	}

	pub fn from_parts(
		id: Id,
		label: Option<String>,
		ty: MetaOption<Ref<ty::Definition<M>>, M>,
		property: MetaOption<Ref<prop::Definition<M>>, M>,
		layout: MetaOption<Ref<layout::Definition<M>>, M>,
		doc: Documentation,
	) -> Self {
		Self {
			id,
			label,
			ty,
			property,
			layout,
			doc,
		}
	}

	pub fn id(&self) -> Id {
		self.id
	}

	pub fn label(&self) -> Option<&str> {
		self.label.as_deref()
	}

	pub fn documentation(&self) -> &Documentation {
		&self.doc
	}

	pub fn documentation_mut(&mut self) -> &mut Documentation {
		&mut self.doc
	}

	pub fn is_type(&self) -> bool {
		self.ty.is_some()
	}

	pub fn is_property(&self) -> bool {
		self.property.is_some()
	}

	pub fn is_layout(&self) -> bool {
		self.layout.is_some()
	}

	pub fn as_type(&self) -> Option<Ref<ty::Definition<M>>> {
		self.ty.value().cloned()
	}

	pub fn as_property(&self) -> Option<Ref<prop::Definition<M>>> {
		self.property.value().cloned()
	}

	pub fn as_layout(&self) -> Option<Ref<layout::Definition<M>>> {
		self.layout.value().cloned()
	}

	pub fn types_metadata(&self) -> TypesMetadata<&M> {
		TypesMetadata {
			ty: self.ty.metadata(),
			property: self.property.metadata(),
			layout: self.layout.metadata(),
			layout_field: None,
			layout_variant: None,
			list: None,
		}
	}

	pub fn require_layout(&self) -> Result<Ref<layout::Definition<M>>, Error<M>>
	where
		M: Clone,
	{
		self.as_layout().ok_or_else(|| {
			error::NodeInvalidType {
				id: self.id,
				expected: Type::Layout,
				found: self.types_metadata().cloned(),
			}
			.into()
		})
	}
}
